use eo::{
    data::{EOChar, EOShort, EOThree},
    protocol::{server::attack, Direction, PacketAction, PacketFamily},
    pubs::EnfNpcType,
};
use rand::Rng;

use crate::{utils::get_next_coords, NPC_DB};

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

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.hidden {
            self.send_packet_near_player(
                player_id,
                PacketAction::Player,
                PacketFamily::Attack,
                reply,
            );
        }

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
            self.attack_npc_reply(player_id, npc_index, direction, damage_dealt, None);
        } else {
            self.attack_npc_killed_reply(player_id, npc_index, direction, damage_dealt, None);
        }
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
