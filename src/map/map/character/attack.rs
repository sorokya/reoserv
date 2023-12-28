use eolib::{protocol::{Direction, net::{server::AttackPlayerServerPacket, PacketAction, PacketFamily}, Coords, r#pub::{NpcType, ItemSubtype}}, data::EoWriter};
use rand::Rng;

use crate::{
    character::Character,
    map::map::ArenaPlayer,
    utils::{get_distance, get_next_coords},
    ITEM_DB, NPC_DB, SETTINGS,
};

use super::super::Map;

enum AttackTarget {
    Npc(i32),
    Player(i32),
}

impl Map {
    // TODO: enforce timestamp
    pub async fn attack(&mut self, player_id: i32, direction: Direction, _timestamp: i32) {
        let reply = AttackPlayerServerPacket {
            player_id,
            direction,
        };

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !can_attack(character) {
            return;
        }

        if !character.hidden {
            self.send_packet_near_player(
                player_id,
                PacketAction::Player,
                PacketFamily::Attack,
                reply,
            );
        }

        match self.get_attack_target(player_id, direction) {
            Some(AttackTarget::Npc(npc_index)) => {
                self.attack_npc(player_id, npc_index, direction).await
            }
            Some(AttackTarget::Player(target_player_id)) => {
                self.attack_player(player_id, target_player_id, direction)
            }
            None => {}
        };
    }

    fn get_attack_target(&self, player_id: i32, direction: Direction) -> Option<AttackTarget> {
        let attacker = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return None,
        };

        let range = get_weapon_range(attacker);
        let mut target_coords: Vec<Coords> = Vec::with_capacity(range as usize);
        for _ in 0..range {
            let next_coords = get_next_coords(
                if target_coords.is_empty() {
                    &attacker.coords
                } else {
                    target_coords.last().unwrap()
                },
                direction,
                self.file.width,
                self.file.height,
            );

            if !self.is_tile_walkable(&next_coords) {
                break;
            }

            if !target_coords.contains(&next_coords) {
                target_coords.push(next_coords);
            }
        }

        target_coords.retain(|c| c != &attacker.coords);

        for coords in target_coords {
            if let Some((index, _)) = self
                .npcs
                .iter()
                .find(|(_, npc)| npc.alive && npc.coords == coords)
            {
                return Some(AttackTarget::Npc(*index));
            }

            if let Some((player_id, _)) = self
                .characters
                .iter()
                .find(|(_, character)| !character.hidden && character.coords == coords)
            {
                return Some(AttackTarget::Player(*player_id));
            };
        }

        None
    }

    async fn attack_npc(&mut self, player_id: i32, npc_index: i32, direction: Direction) {
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
            NpcType::Passive | NpcType::Aggressive
        ) {
            return;
        }

        let amount = {
            let mut rng = rand::thread_rng();
            rng.gen_range(attacker.min_damage..=attacker.max_damage)
        };

        let attacker_facing_npc =
            (i32::from(npc.direction) - i32::from(attacker.direction)).abs() != 2;

        let critical = npc.hp == npc.max_hp || attacker_facing_npc;

        let damage_dealt = npc.damage(player_id, amount, attacker.accuracy, critical);

        if npc.alive {
            self.attack_npc_reply(player_id, npc_index, direction, damage_dealt, None);
        } else {
            self.attack_npc_killed_reply(player_id, npc_index, direction, damage_dealt, None)
                .await;
        }
    }

    fn attack_player(
        &mut self,
        player_id: i32,
        target_player_id: i32,
        direction: Direction,
    ) {
        if self.arena_players.iter().any(|p| p.player_id == player_id) {
            return self.attack_player_arena(player_id, target_player_id, direction);
        }

        error!("PVP is not implemented yet!");
    }

    fn attack_player_arena(
        &mut self,
        player_id: i32,
        target_player_id: i32,
        direction: Direction,
    ) {
        if !self
            .arena_players
            .iter()
            .any(|p| p.player_id == target_player_id)
        {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if get_distance(&character.coords, &target_character.coords) > 1 {
            return;
        }

        let arena_player = self
            .arena_players
            .iter_mut()
            .find(|p| p.player_id == player_id)
            .unwrap();

        arena_player.kills += 1;

        let arena_player = arena_player.to_owned();

        target_character.player.as_ref().unwrap().arena_die(Coords {
            x: self.file.relog_x,
            y: self.file.relog_y,
        });

        self.arena_players
            .retain(|p| p.player_id != target_player_id);

        if self.arena_players.len() == 1 {
            character.player.as_ref().unwrap().arena_die(Coords {
                x: self.file.relog_x,
                y: self.file.relog_y,
            });

            return self.arena_end(
                &arena_player,
                character.name.to_owned(),
                target_character.name.to_owned(),
            );
        }

        let mut writer = EoWriter::new();
        writer.add_short(player_id);
        writer.add_byte(0xff);
        writer.add_char(i32::from(direction));
        writer.add_byte(0xff);
        writer.add_short(arena_player.kills);
        writer.add_byte(0xff);
        writer.add_string(&character.name);
        writer.add_byte(0xff);
        writer.add_string(&target_character.name);

        let buf = writer.to_byte_array();

        for character in self.characters.values() {
            character.player.as_ref().unwrap().send(
                PacketAction::Spec,
                PacketFamily::Arena,
                buf.clone(),
            );
        }
    }

    fn arena_end(&mut self, arena_player: &ArenaPlayer, winner_name: String, target_name: String) {
        self.arena_players.clear();

        let mut writer = EoWriter::new();
        writer.add_string(&winner_name);
        writer.add_byte(0xff);
        writer.add_short(arena_player.kills);
        writer.add_byte(0xff);
        writer.add_string(&winner_name);
        writer.add_byte(0xff);
        writer.add_string(&target_name);

        let buf = writer.to_byte_array();

        for character in self.characters.values() {
            character.player.as_ref().unwrap().send(
                PacketAction::Accept,
                PacketFamily::Arena,
                buf.clone(),
            );
        }
    }
}

fn can_attack(character: &Character) -> bool {
    let weapon = character.paperdoll.weapon;
    let shield = character.paperdoll.shield;

    if weapon == 0 {
        return true;
    }

    if let Some(config) = SETTINGS
        .combat
        .weapon_ranges
        .iter()
        .find(|s| s.weapon == weapon)
    {
        if !config.arrows {
            return true;
        }

        let shield_data = match ITEM_DB.items.get(shield as usize - 1) {
            Some(data) => data,
            None => return false,
        };

        return shield_data.subtype == ItemSubtype::Arrows;
    }

    true
}

fn get_weapon_range(character: &Character) -> i32 {
    let weapon = character.paperdoll.weapon;
    if weapon == 0 {
        return 1;
    }

    if let Some(config) = SETTINGS
        .combat
        .weapon_ranges
        .iter()
        .find(|s| s.weapon == weapon)
    {
        return config.range;
    }

    1
}
