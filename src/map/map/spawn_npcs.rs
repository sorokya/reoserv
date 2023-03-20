use std::cmp;

use chrono::Duration;
use eo::{data::EOChar, protocol::{Coords, Direction}, pubs::EnfNpc};
use rand::Rng;

use crate::{SETTINGS, map::{Npc, NpcData, is_tile_walkable::is_tile_walkable_for_npc}};

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

                    let data_record = match self.world.get_npc(spawn.id).await {
                            Ok(npc) =>  npc,
                            Err(e) => {
                                error!("Failed to load NPC {}", e);
                                continue;
                            }
                    };

                    // TODO: bounds check
                    for _ in 0..spawn.amount {
                        self.npcs.insert(
                            npc_index,
                            Npc::new(
                                spawn.id,
                                Coords::default(),
                                Direction::Down,
                                spawn_index,
                                dead_since,
                                dead_since,
                                chatter_indexes.contains(&(npc_index as usize)),
                                now,
                                data_record.hp,
                            ),
                        );
                        npc_index += 1;
                    }

                    self.npc_data.entry(spawn.id).or_insert({
                        let data_record = match self.world.get_npc(spawn.id).await {
                            Ok(npc) => Some(npc),
                            Err(e) => {
                                error!("Failed to load NPC {}", e);
                                None
                            }
                        };

                        if data_record.is_some() {
                            let drop_record = self.world.get_drop_record(spawn.id).await;
                            let talk_record = self.world.get_talk_record(spawn.id).await;
                            NpcData {
                                npc_record: data_record.unwrap(),
                                drop_record,
                                talk_record,
                            }
                        } else {
                            warn!("Map {} has NPC {} but no NPC record", self.id, spawn.id);
                            NpcData {
                                npc_record: EnfNpc::default(),
                                drop_record: None,
                                talk_record: None,
                            }
                        }
                    });
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