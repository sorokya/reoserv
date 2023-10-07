use eo::{
    data::{EOChar, EOInt, EOShort, EOThree},
    protocol::{
        server::{attack, npc},
        Direction, LevelUpStats, PacketAction, PacketFamily,
    },
    pubs::EnfNpcType,
};
use rand::Rng;

use crate::{
    map::{Item, Npc},
    utils::get_next_coords,
    DROP_DB, NPC_DB,
};

use super::Map;

enum AttackTarget {
    Npc(EOChar),
    Player(EOShort),
}

impl Map {
    // TODO: enforce timestamp
    pub fn attack(&mut self, player_id: EOShort, direction: Direction, _timestamp: EOThree) {
        let reply = attack::Player {
            player_id,
            direction,
        };

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Attack, reply);

        match self.get_attack_target(player_id, direction) {
            Some(AttackTarget::Npc(npc_index)) => self.attack_npc(player_id, npc_index, direction),
            Some(AttackTarget::Player(target_player_id)) => {
                self.attack_player(player_id, target_player_id, direction)
            }
            None => {}
        };
    }

    fn get_attack_target(&self, player_id: EOShort, direction: Direction) -> Option<AttackTarget> {
        let attacker = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return None,
        };

        let target_coords = get_next_coords(
            &attacker.coords,
            direction,
            self.file.width,
            self.file.height,
        );
        if target_coords == attacker.coords {
            return None;
        }

        if let Some((index, _)) = self
            .npcs
            .iter()
            .find(|(_, npc)| npc.coords == target_coords && npc.alive)
        {
            return Some(AttackTarget::Npc(*index));
        }

        if let Some((player_id, _)) = self
            .characters
            .iter()
            .find(|(_, character)| character.coords == target_coords)
        {
            return Some(AttackTarget::Player(*player_id));
        };

        None
    }

    fn attack_npc(&mut self, player_id: EOShort, npc_index: EOChar, direction: Direction) {
        let attacker = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc = match self.npcs.get_mut(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if !matches!(
            npc_data.r#type,
            EnfNpcType::Passive | EnfNpcType::Aggressive
        ) {
            return;
        }

        let mut rng = rand::thread_rng();

        let amount = rng.gen_range(attacker.min_damage..=attacker.max_damage);

        let attacker_facing_npc =
            ((npc.direction.to_char() as i32) - (attacker.direction.to_char() as i32)).abs() != 2;

        let critical = npc.hp == npc.max_hp || attacker_facing_npc;

        let damage_dealt = npc.damage(player_id, amount, attacker.accuracy, critical);

        if npc.alive {
            self.attack_npc_reply(player_id, npc_index, direction, damage_dealt);
        } else {
            self.attack_npc_killed_reply(player_id, npc_index, direction, damage_dealt);
        }
    }

    fn attack_npc_reply(
        &mut self,
        player_id: EOShort,
        npc_index: EOChar,
        direction: Direction,
        damage_dealt: EOInt,
    ) {
        let reply = attack::Player {
            player_id,
            direction,
        };

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Attack, reply);

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let reply = npc::Reply {
            player_id,
            npc_index: npc_index as EOShort,
            damage: damage_dealt,
            player_direction: direction.to_char(),
            hp_percentage: npc.get_hp_percentage() as EOShort,
        };

        self.send_packet_near(&npc.coords, PacketAction::Reply, PacketFamily::Npc, reply);
    }

    fn attack_npc_killed_reply(
        &mut self,
        player_id: EOShort,
        npc_index: EOChar,
        direction: Direction,
        damage_dealt: EOInt,
    ) {
        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let leveled_up = character.add_experience(npc_data.experience);

        let drop = { get_drop(player_id, npc) };

        if leveled_up {
            self.attack_npc_killed_leveled_up_reply(
                player_id,
                npc_index,
                direction,
                damage_dealt,
                drop,
            );
        } else {
            let mut packet = npc::Spec {
                killer_id: player_id,
                killer_direction: direction.to_char(),
                npc_index: npc_index as EOShort,
                damage: damage_dealt,
                experience: character.experience,
                ..Default::default()
            };

            if let Some(drop) = drop {
                let index = self.get_next_item_index(1);
                packet.drop_index = index;
                packet.drop_id = drop.id;
                packet.drop_coords = drop.coords;
                packet.drop_amount = drop.amount;
                self.items.insert(index, drop);
            }

            self.send_packet_near(&npc.coords, PacketAction::Spec, PacketFamily::Npc, packet);
        }
    }

    fn attack_npc_killed_leveled_up_reply(
        &mut self,
        player_id: EOShort,
        npc_index: EOChar,
        direction: Direction,
        damage_dealt: EOInt,
        drop: Option<Item>,
    ) {
        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let mut packet = npc::Accept {
            killer_id: player_id,
            killer_direction: direction.to_char(),
            npc_index: npc_index as EOShort,
            damage: damage_dealt,
            experience: character.experience,
            level_up: LevelUpStats {
                level: character.level,
                stat_points: character.stat_points,
                skill_points: character.skill_points,
                max_hp: character.max_hp,
                max_tp: character.max_tp,
                max_sp: character.max_sp,
            },
            ..Default::default()
        };

        if let Some(drop) = drop {
            let index = self.get_next_item_index(1);
            packet.drop_index = index;
            packet.drop_id = drop.id;
            packet.drop_coords = drop.coords;
            packet.drop_amount = drop.amount;
            self.items.insert(index, drop);
        }

        self.send_packet_near(&npc.coords, PacketAction::Accept, PacketFamily::Npc, packet);
    }

    fn attack_player(
        &mut self,
        _player_id: EOShort,
        _target_player_id: EOShort,
        _direction: Direction,
    ) {
        error!("PVP is not implemented yet!");
    }
}

fn get_drop(target_player_id: EOShort, npc: &Npc) -> Option<Item> {
    if let Some(drop_npc) = DROP_DB.npcs.iter().find(|d| d.npc_id == npc.id) {
        let mut rng = rand::thread_rng();
        let mut drops = drop_npc.drops.clone();
        drops.sort_by(|a, b| a.rate.cmp(&b.rate));

        for drop in drops {
            let roll = rng.gen_range(0..=64000);
            if roll <= drop.rate {
                let amount = rng.gen_range(drop.min..=drop.max);
                return Some(Item {
                    id: drop.item_id,
                    amount,
                    coords: npc.coords,
                    owner: target_player_id,
                });
            }
        }
    }

    None
}
