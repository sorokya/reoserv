use std::cmp;

use chrono::{Duration, Utc};
use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::npc, Coords, Direction, NPCUpdateAttack, NPCUpdateChat, NPCUpdatePos, PacketAction,
        PacketFamily, PlayerKilledState, SitState,
    },
    pubs::{EnfNpc, EnfNpcType},
};
use evalexpr::{context_map, eval_float_with_context};
use rand::Rng;

use crate::{character::Character, map::Npc, FORMULAS, NPC_DB, SETTINGS, TALK_DB};

use super::Map;

impl Map {
    fn act_npc_talk(&mut self, index: EOChar, npc_id: EOShort) -> Option<NPCUpdateChat> {
        let talk_record = match TALK_DB.npcs.iter().find(|record| record.npc_id == npc_id) {
            Some(record) => record,
            None => return None,
        };

        let now = Utc::now();
        let mut rng = rand::thread_rng();

        let npc = match self.npcs.get_mut(&index) {
            Some(npc) => npc,
            None => return None,
        };

        let talk_delta = now - npc.last_talk.unwrap();
        if !npc.alive || talk_delta < Duration::milliseconds(SETTINGS.npcs.talk_rate as i64) {
            return None;
        }

        npc.last_talk = Some(now);
        let roll = rng.gen_range(0..=100);
        if roll <= talk_record.rate {
            let message_index = rng.gen_range(0..talk_record.messages.len());
            Some(NPCUpdateChat {
                npc_index: index,
                message_length: talk_record.messages[message_index].len() as EOChar,
                message: talk_record.messages[message_index].to_string(),
            })
        } else {
            None
        }
    }

    fn act_npc_move_chase(&mut self, index: EOChar, npc_id: EOShort) -> Option<NPCUpdatePos> {
        None
    }

    fn act_npc_move_idle(&mut self, index: EOChar) -> Option<NPCUpdatePos> {
        let (direction, coords) = match self.npcs.get(&index) {
            Some(npc) => (npc.direction, npc.coords),
            None => return None,
        };
        // Logic ripped from EOServ..
        let mut rng = rand::thread_rng();
        let action = rng.gen_range(1..=10);

        if action == 10 {
            self.npcs.get_mut(&index).unwrap().walk_idle_for =
                Some(Duration::seconds(rng.gen_range(1..=4)));
            return None;
        }

        let new_direction = if (7..=9).contains(&action) {
            Direction::from_char(rng.gen_range(0..=3)).unwrap()
        } else {
            direction
        };

        let new_coords = match new_direction {
            Direction::Down => {
                if coords.y >= self.file.height {
                    coords
                } else {
                    Coords {
                        x: coords.x,
                        y: coords.y + 1,
                    }
                }
            }
            Direction::Left => {
                if coords.x == 0 {
                    coords
                } else {
                    Coords {
                        x: coords.x - 1,
                        y: coords.y,
                    }
                }
            }
            Direction::Up => {
                if coords.y == 0 {
                    coords
                } else {
                    Coords {
                        x: coords.x,
                        y: coords.y - 1,
                    }
                }
            }
            Direction::Right => {
                if coords.x >= self.file.width {
                    coords
                } else {
                    Coords {
                        x: coords.x + 1,
                        y: coords.y,
                    }
                }
            }
        };

        if let Some(npc) = self.npcs.get_mut(&index) {
            npc.direction = new_direction;
            npc.last_act = Some(Utc::now());
            npc.walk_idle_for = None;
        }

        if self.is_tile_walkable_npc(&new_coords) {
            if let Some(npc) = self.npcs.get_mut(&index) {
                npc.coords = new_coords;
            }

            Some(NPCUpdatePos {
                npc_index: index,
                coords: new_coords,
                direction: new_direction,
            })
        } else {
            None
        }
    }

    fn act_npc_move(
        &mut self,
        index: EOChar,
        npc_id: EOShort,
        npc_data: &EnfNpc,
        act_rate: EOInt,
        act_delta: &Duration,
    ) -> Option<NPCUpdatePos> {
        let (walk_idle_for, has_oppenents) = {
            match self.npcs.get(&index) {
                Some(npc) => (
                    npc.walk_idle_for.unwrap_or_else(|| Duration::seconds(0)),
                    !npc.oppenents.is_empty(),
                ),
                None => return None,
            }
        };

        let idle_rate = Duration::milliseconds(act_rate as i64) + walk_idle_for;

        if npc_data.r#type == EnfNpcType::Aggressive || has_oppenents {
            self.act_npc_move_chase(index, npc_id)
        } else if act_delta >= &idle_rate {
            self.act_npc_move_idle(index)
        } else {
            None
        }
    }

    fn act_npc_attack(&mut self, index: EOChar, npc_data: &EnfNpc) -> Option<NPCUpdateAttack> {
        let (opponent_id, direction) = match self.npcs.get(&index) {
            Some(npc) => {
                if npc.oppenents.is_empty() {
                    return None;
                } else {
                    let adjacent_tiles = self.get_adjacent_tiles(&npc.coords);
                    let adjacent_opponents = npc
                        .oppenents
                        .iter()
                        .filter(|(player_id, _)| match self.characters.get(player_id) {
                            Some(character) => adjacent_tiles.contains(&character.coords),
                            None => false,
                        })
                        .collect::<Vec<_>>();

                    // get the adjacent component with the most damage dealt
                    let opponent_id =
                        match adjacent_opponents.iter().max_by_key(|(_, damage)| *damage) {
                            Some((opponent_id, _)) => opponent_id,
                            None => return None,
                        };

                    let opponent_coords = match self.characters.get(opponent_id) {
                        Some(character) => character.coords,
                        None => return None,
                    };

                    let xdiff = npc.coords.x as i32 - opponent_coords.x as i32;
                    let ydiff = npc.coords.y as i32 - opponent_coords.y as i32;

                    let direction = match (xdiff, ydiff) {
                        (0, 1) => Direction::Up,
                        (0, -1) => Direction::Down,
                        (1, 0) => Direction::Left,
                        (-1, 0) => Direction::Right,
                        _ => return None,
                    };

                    (**opponent_id, direction)
                }
            }
            None => return None,
        };

        let damage = {
            let character = match self.characters.get(&opponent_id) {
                Some(character) => character,
                None => return None,
            };

            let npc = match self.npcs.get(&index) {
                Some(npc) => npc,
                None => return None,
            };

            get_damage_amount(npc, npc_data, character)
        };

        let (killed_state, hp_percentage) = {
            let character = match self.characters.get_mut(&opponent_id) {
                Some(character) => character,
                None => return None,
            };

            character.hp -= damage as EOShort;

            let killed_state = if character.hp == 0 {
                PlayerKilledState::Dead
            } else {
                PlayerKilledState::Alive
            };

            (killed_state, character.get_hp_percentage())
        };

        if let Some(npc) = self.npcs.get_mut(&index) {
            npc.direction = direction;
            npc.last_act = Some(Utc::now());
        }

        Some(NPCUpdateAttack {
            npc_index: index,
            killed_state,
            direction,
            player_id: opponent_id,
            damage,
            hp_percentage,
        })
    }

    fn act_npc(
        &mut self,
        index: EOChar,
    ) -> (
        Option<NPCUpdatePos>,
        Option<NPCUpdateChat>,
        Option<NPCUpdateAttack>,
    ) {
        let (npc_id, spawn_index, last_act) = match self.npcs.get(&index) {
            Some(npc) => {
                if !npc.alive {
                    return (None, None, None);
                } else {
                    (npc.id, npc.spawn_index, npc.last_act.unwrap())
                }
            }
            None => return (None, None, None),
        };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc) => npc,
            None => return (None, None, None),
        };

        let spawn = &self.file.npcs[spawn_index];
        let act_rate = match spawn.spawn_type {
            0 => SETTINGS.npcs.speed_0,
            1 => SETTINGS.npcs.speed_1,
            2 => SETTINGS.npcs.speed_2,
            3 => SETTINGS.npcs.speed_3,
            4 => SETTINGS.npcs.speed_4,
            5 => SETTINGS.npcs.speed_5,
            6 => SETTINGS.npcs.speed_6,
            7 => 0,
            _ => unreachable!("Invalid spawn type {} for NPC {}", spawn.spawn_type, npc_id),
        };

        let talk_update = self.act_npc_talk(index, npc_id);

        let now = Utc::now();
        let act_delta = now - last_act;
        if act_rate == 0 || act_delta < Duration::milliseconds(act_rate as i64) {
            (None, talk_update, None)
        } else {
            let pos_update = self.act_npc_move(index, npc_id, npc_data, act_rate, &act_delta);
            let attack_update = self.act_npc_attack(index, npc_data);
            (pos_update, talk_update, attack_update)
        }
    }

    pub fn act_npcs(&mut self) {
        if self.npcs.is_empty() || SETTINGS.npcs.freeze_on_empty_map && self.characters.is_empty() {
            return;
        }

        let now = Utc::now();

        if self.npcs.get(&0).unwrap().last_act.is_none() {
            for (spawn_index, spawn) in self.file.npcs.iter().enumerate() {
                let npcs = {
                    self.npcs
                        .iter()
                        .filter(|(_, npc)| npc.spawn_index == spawn_index && npc.id == spawn.id)
                        .map(|(index, _)| *index)
                        .collect::<Vec<EOChar>>()
                        .clone()
                };

                for index in npcs {
                    let npc = self.npcs.get_mut(&index).unwrap();
                    npc.last_act = Some(now);
                    npc.last_talk = Some(now + Duration::milliseconds(7500 * index as i64));
                }
            }
        }

        let mut attack_updates: Vec<NPCUpdateAttack> = Vec::with_capacity(self.npcs.len());
        let mut position_updates: Vec<NPCUpdatePos> = Vec::with_capacity(self.npcs.len());
        let mut talk_updates: Vec<NPCUpdateChat> = Vec::with_capacity(self.npcs.len());

        let indexes = self.npcs.keys().cloned().collect::<Vec<EOChar>>();
        for index in indexes {
            let (move_update, chat_updatee, attack_update) = self.act_npc(index);
            if let Some(attack_update) = attack_update {
                attack_updates.push(attack_update);
            }
            if let Some(move_update) = move_update {
                position_updates.push(move_update);
            }
            if let Some(chat_update) = chat_updatee {
                talk_updates.push(chat_update);
            }
        }

        if !position_updates.is_empty() || !attack_updates.is_empty() || !talk_updates.is_empty() {
            for character in self.characters.values() {
                // TODO: might also need to check NPCs previous position..

                let in_range_npc_indexes: Vec<EOChar> = self
                    .npcs
                    .iter()
                    .filter(|(_, n)| n.is_in_range(&character.coords))
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

                let attack_updates_in_range: Vec<NPCUpdateAttack> = attack_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                if !position_updates_in_rage.is_empty()
                    || !talk_updates_in_range.is_empty()
                    || !attack_updates_in_range.is_empty()
                {
                    let packet = npc::Player {
                        pos: position_updates_in_rage,
                        attack: attack_updates_in_range,
                        chat: talk_updates_in_range,
                    };

                    debug!("Send: {:?}", packet);

                    let mut builder = StreamBuilder::new();
                    packet.serialize(&mut builder);

                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Npc,
                        builder.get(),
                    );

                    if let Some(player_id) = character.player_id {
                        let player_died = packet.attack.iter().any(|update| {
                            update.player_id == player_id
                                && update.killed_state == PlayerKilledState::Dead
                        });

                        if player_died {
                            character.player.as_ref().unwrap().die();
                        }
                    }
                }
            }
        }
    }

    fn get_adjacent_tiles(&self, coords: &Coords) -> Vec<Coords> {
        let mut adjacent_tiles = Vec::with_capacity(4);
        adjacent_tiles.push(Coords {
            x: coords.x,
            y: cmp::max(coords.y as i32 - 1, 0) as EOChar,
        });
        adjacent_tiles.push(Coords {
            x: coords.x,
            y: cmp::min(coords.y as i32 + 1, self.file.height as i32) as EOChar,
        });
        adjacent_tiles.push(Coords {
            x: cmp::max(coords.x as i32 - 1, 0) as EOChar,
            y: coords.y,
        });
        adjacent_tiles.push(Coords {
            x: cmp::min(coords.x as i32 + 1, self.file.width as i32) as EOChar,
            y: coords.y,
        });

        adjacent_tiles.dedup();
        adjacent_tiles
    }
}

fn get_damage_amount(npc: &Npc, npc_data: &EnfNpc, character: &Character) -> EOInt {
    let mut rng = rand::thread_rng();
    let rand = rng.gen_range(0.0..=1.0);

    let amount = rng.gen_range(npc_data.min_damage..=npc_data.max_damage);

    let npc_facing_player_back_or_side =
        ((character.direction.to_char() as i32) - (npc.direction.to_char() as i32)).abs() != 2;

    let context = match context_map! {
        "critical" => npc_facing_player_back_or_side,
        "damage" => amount as f64,
        "target_armor" => character.armor as f64,
        "target_sitting" => character.sit_state != SitState::Stand,
        "accuracy" => npc_data.accuracy as f64,
        "target_evade" => character.evasion as f64,
    } {
        Ok(context) => context,
        Err(e) => {
            error!("Failed to generate formula context: {}", e);
            return 0;
        }
    };

    let hit_rate = match eval_float_with_context(&FORMULAS.hit_rate, &context) {
        Ok(hit_rate) => hit_rate,
        Err(e) => {
            error!("Failed to calculate hit rate: {}", e);
            0.0
        }
    };

    if hit_rate < rand {
        return 0;
    }

    match eval_float_with_context(&FORMULAS.damage, &context) {
        Ok(amount) => cmp::min(amount.floor() as EOInt, character.hp as EOInt),
        Err(e) => {
            error!("Failed to calculate damage: {}", e);
            0
        }
    }
}
