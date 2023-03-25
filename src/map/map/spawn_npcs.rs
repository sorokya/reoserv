use std::cmp;

use chrono::Duration;
use eo::{
    data::EOChar,
    protocol::{Coords, Direction},
};
use rand::Rng;

use crate::{
    map::{is_tile_walkable::is_tile_walkable_for_npc, NPCBuilder},
    NPC_DB, SETTINGS,
};

use super::Map;

impl Map {
    pub async fn spawn_npcs(&mut self) {
        // TODO: test if this is actually how GameServer.exe works
        let now = chrono::Utc::now();

        if !self.file.npcs.is_empty() {
            if self.npcs.is_empty() {
                let mut npc_index: EOChar = 0;

                let dead_since = if SETTINGS.npcs.instant_spawn {
                    now - Duration::days(1)
                } else {
                    now
                };

                for (spawn_index, spawn) in self.file.npcs.iter().enumerate() {
                    // Only 20% of npcs in a group will speak
                    let num_of_chatters =
                        cmp::max(1, (spawn.amount as f64 * 0.2).floor() as EOChar);
                    let mut chatter_indexes: Vec<usize> =
                        Vec::with_capacity(num_of_chatters as usize);
                    let chatter_distribution = spawn.amount / num_of_chatters;
                    for i in 0..num_of_chatters {
                        chatter_indexes.push(((i * chatter_distribution) + npc_index) as usize);
                    }

                    let data_record = match NPC_DB.npcs.get(spawn.id as usize - 1) {
                        Some(npc) => npc,
                        None => {
                            error!("Failed to load NPC {}", spawn.id);
                            continue;
                        }
                    };

                    // TODO: bounds check
                    for _ in 0..spawn.amount {
                        self.npcs.insert(
                            npc_index,
                            NPCBuilder::new()
                                .id(spawn.id)
                                .coords(Coords::default())
                                .direction(Direction::Down)
                                .spawn_index(spawn_index)
                                .alive(false)
                                .dead_since(dead_since)
                                .last_act(dead_since)
                                .does_talk(chatter_indexes.contains(&(npc_index as usize)))
                                .last_talk(now)
                                .hp(data_record.hp)
                                .max_hp(data_record.hp)
                                .build(),
                        );
                        npc_index += 1;
                    }
                }
            }

            let mut rng = rand::thread_rng();
            for npc in self.npcs.values_mut() {
                let spawn = &self.file.npcs[npc.spawn_index];
                if !npc.alive
                    && now.timestamp() - npc.dead_since.timestamp() > spawn.spawn_time.into()
                {
                    npc.alive = true;
                    npc.coords = Coords {
                        x: spawn.x,
                        y: spawn.y,
                    };

                    while !is_tile_walkable_for_npc(
                        npc.coords,
                        &self.file.spec_rows,
                        &self.file.warp_rows,
                    ) {
                        npc.coords.x += cmp::max(rng.gen_range(-1..=1), 0) as EOChar;
                        npc.coords.y += cmp::max(rng.gen_range(-1..=1), 0) as EOChar;
                    }

                    npc.direction = if spawn.spawn_type == 7 {
                        Direction::from_char(spawn.spawn_type & 0x03).unwrap()
                    } else {
                        match rand::random::<u8>() % 4 {
                            0 => Direction::Down,
                            1 => Direction::Left,
                            2 => Direction::Up,
                            3 => Direction::Right,
                            _ => unreachable!(),
                        }
                    };
                }
            }
        }
    }
}
