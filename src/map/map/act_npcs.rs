use chrono::{Duration, Utc};
use eo::{
    data::{EOChar, Serializeable},
    protocol::{
        server::npc, Coords, Direction, NPCUpdateChat, NPCUpdatePos, PacketAction, PacketFamily,
    },
};
use rand::Rng;

use crate::{
    map::{is_occupied, is_tile_walkable::is_tile_walkable_for_npc},
    SETTINGS,
};

use super::Map;

impl Map {
    pub fn act_npcs(&mut self) {
        if SETTINGS.npcs.freeze_on_empty_map && self.characters.is_empty() {
            return;
        }

        let now = Utc::now();

        let mut rng = rand::thread_rng();

        // get occupied tiles of all characters and npcs
        let mut occupied_tiles = Vec::new();
        for character in self.characters.values() {
            occupied_tiles.push(character.coords);
        }
        for npc in self.npcs.values() {
            occupied_tiles.push(npc.coords);
        }

        let mut position_updates: Vec<NPCUpdatePos> = Vec::with_capacity(self.npcs.len());
        let mut talk_updates: Vec<NPCUpdateChat> = Vec::with_capacity(self.npcs.len());

        for (index, npc) in &mut self.npcs {
            let spawn = &self.file.npcs[npc.spawn_index];
            let act_rate = match spawn.spawn_type {
                0 => SETTINGS.npcs.speed_0,
                1 => SETTINGS.npcs.speed_1,
                2 => SETTINGS.npcs.speed_2,
                3 => SETTINGS.npcs.speed_3,
                4 => SETTINGS.npcs.speed_4,
                5 => SETTINGS.npcs.speed_5,
                6 => SETTINGS.npcs.speed_6,
                7 => 0,
                _ => unreachable!("Invalid spawn type {} for NPC {}", spawn.spawn_type, npc.id),
            };

            let act_delta = now - npc.last_act;
            let walk_idle_for_ms = if let Some(walk_idle_for) = npc.walk_idle_for {
                walk_idle_for.num_milliseconds()
            } else {
                0
            };

            if npc.alive
                && act_rate > 0
                && act_delta >= Duration::milliseconds(act_rate as i64 + walk_idle_for_ms)
            {
                // TODO: attack

                // Logic ripped from EOServ..
                let action = rng.gen_range(1..=10);
                if (7..=9).contains(&action) {
                    npc.direction = Direction::from_char(rng.gen_range(0..=3)).unwrap();
                }

                if action != 10 {
                    let new_coords = match npc.direction {
                        Direction::Down => {
                            if npc.coords.y >= self.file.height {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x,
                                    y: npc.coords.y + 1,
                                }
                            }
                        }
                        Direction::Left => {
                            if npc.coords.x == 0 {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x - 1,
                                    y: npc.coords.y,
                                }
                            }
                        }
                        Direction::Up => {
                            if npc.coords.y == 0 {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x,
                                    y: npc.coords.y - 1,
                                }
                            }
                        }
                        Direction::Right => {
                            if npc.coords.x >= self.file.width {
                                npc.coords
                            } else {
                                Coords {
                                    x: npc.coords.x + 1,
                                    y: npc.coords.y,
                                }
                            }
                        }
                    };

                    if !is_occupied(new_coords, &occupied_tiles)
                        && is_tile_walkable_for_npc(
                            new_coords,
                            &self.file.spec_rows,
                            &self.file.warp_rows,
                        )
                    {
                        // TODO: Fix if multiple npcs or players are on the same tile
                        // Should only remove one at the most
                        occupied_tiles.retain(|coords| *coords != npc.coords);
                        npc.coords = new_coords;
                        position_updates.push(NPCUpdatePos {
                            npc_index: *index,
                            coords: npc.coords,
                            direction: npc.direction,
                        });
                        occupied_tiles.push(new_coords);
                    }

                    npc.last_act = Utc::now();
                    npc.walk_idle_for = None;
                } else {
                    npc.walk_idle_for = Some(Duration::seconds(rng.gen_range(1..=4)));
                }
            }

            if let Some(npc_data) = self.npc_data.get(&npc.id) {
                if let Some(talk_record) = &npc_data.talk_record {
                    let talk_delta = now - npc.last_talk;
                    if npc.alive
                        && npc.does_talk
                        && talk_delta >= Duration::milliseconds(SETTINGS.npcs.talk_rate as i64)
                    {
                        let roll = rng.gen_range(0..=100);
                        if roll <= talk_record.rate {
                            let message_index = rng.gen_range(0..talk_record.messages.len());
                            talk_updates.push(NPCUpdateChat {
                                npc_index: *index,
                                message_length: talk_record.messages[message_index].len() as EOChar,
                                message: talk_record.messages[message_index].to_string(),
                            })
                        }
                        npc.last_talk = now;
                    }
                }
            }
        }

        if !position_updates.is_empty() || !talk_updates.is_empty() {
            for character in self.characters.values() {
                // TODO: might also need to check NPCs previous position..

                let in_range_npc_indexes: Vec<EOChar> = self
                    .npcs
                    .iter()
                    .filter(|(_, n)| n.is_in_range(character.coords))
                    .map(|(i, _)| i)
                    .cloned()
                    .collect();

                let position_updates_in_rage: Vec<NPCUpdatePos> = position_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                let talk_updates_in_range: Vec<NPCUpdateChat> = talk_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                if !position_updates_in_rage.is_empty() || !talk_updates_in_range.is_empty() {
                    let packet = npc::Player {
                        pos: position_updates_in_rage,
                        attack: Vec::new(),
                        chat: talk_updates_in_range,
                    };

                    debug!("Send: {:?}", packet);
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Npc,
                        packet.serialize(),
                    );
                }
            }
        }
    }
}
