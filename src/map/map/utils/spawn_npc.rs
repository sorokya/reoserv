use std::cmp;

use eolib::{data::CHAR_MAX, protocol::Direction};

use crate::{map::Npc, NPC_DB};

use super::super::Map;

impl Map {
    pub fn spawn_npc(&mut self, player_id: i32, npc_id: i32, amount: i32, speed: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(data) => data,
            None => return,
        };

        let max_index = self.npcs.len() as i32;

        if max_index >= CHAR_MAX {
            return;
        }

        let amount = cmp::min(CHAR_MAX - max_index, amount);

        for i in 0..amount {
            self.npcs.insert(
                max_index + i,
                Npc {
                    id: npc_id,
                    coords: character.coords,
                    direction: Direction::Down,
                    spawn_type: speed,
                    spawn_index: None,
                    alive: true,
                    hp: npc_data.hp,
                    max_hp: npc_data.hp,
                    boss: npc_data.boss,
                    child: npc_data.child,
                    ..Default::default()
                },
            );
        }
    }
}
