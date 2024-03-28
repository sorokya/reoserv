use std::cmp;

use eolib::protocol::{
    net::{
        server::{
            NpcPlayerServerPacket, NpcUpdateAttack, NpcUpdateChat, NpcUpdatePosition,
            PlayerKilledState, SitState,
        },
        PacketAction, PacketFamily,
    },
    r#pub::{EnfRecord, NpcType},
    Coords, Direction,
};
use eoplus::Arg;
use evalexpr::{context_map, eval_float_with_context};
use rand::{seq::SliceRandom, Rng};

use crate::{
    character::Character,
    map::Npc,
    utils::{get_distance, get_next_coords, in_range},
    FORMULAS, NPC_DB, QUEST_DB, SETTINGS, TALK_DB,
};

use super::super::Map;

impl Map {
    fn act_npc_talk(&mut self, index: i32, npc_id: i32) -> Option<NpcUpdateChat> {
        let talk_record = match TALK_DB.npcs.iter().find(|record| record.npc_id == npc_id) {
            Some(record) => record,
            None => return None,
        };

        let npc = match self.npcs.get_mut(&index) {
            Some(npc) => npc,
            None => return None,
        };

        if !npc.alive || npc.talk_ticks < SETTINGS.npcs.talk_rate {
            return None;
        }

        npc.talk_ticks = 0;

        let mut rng = rand::thread_rng();
        let roll = rng.gen_range(0..=100);
        if roll <= talk_record.rate {
            let message_index = rng.gen_range(0..talk_record.messages.len());
            Some(NpcUpdateChat {
                npc_index: index,
                message: talk_record.messages[message_index].message.to_owned(),
            })
        } else {
            None
        }
    }

    fn act_npc_move_chase(
        &mut self,
        index: i32,
        npc_id: i32,
        npc_type: NpcType,
    ) -> Option<NpcUpdatePosition> {
        let target_coords = match self.npc_get_chase_target_coords(index, npc_id) {
            Some(target_coords) => target_coords,
            None => {
                if npc_type == NpcType::Passive {
                    return self.act_npc_move_idle(index);
                } else {
                    return None;
                }
            }
        };

        let npc_coords = match self.npcs.get(&index) {
            Some(npc) => npc.coords,
            None => return None,
        };

        let x_delta = npc_coords.x - target_coords.x;
        let y_delta = npc_coords.y - target_coords.y;

        let mut direction = if x_delta.abs() > y_delta.abs() {
            if x_delta < 0 {
                Direction::Right
            } else {
                Direction::Left
            }
        } else if y_delta < 0 {
            Direction::Down
        } else {
            Direction::Up
        };

        let new_coords = get_next_coords(&npc_coords, direction, self.file.width, self.file.height);

        if self.is_tile_walkable_npc(&new_coords) && !self.is_tile_occupied(&new_coords) {
            let npc = self.npcs.get_mut(&index).unwrap();
            npc.direction = direction;
            npc.coords = new_coords;
            npc.act_ticks = 0;
            Some(NpcUpdatePosition {
                npc_index: index,
                coords: npc.coords,
                direction: npc.direction,
            })
        } else {
            if matches!(direction, Direction::Up | Direction::Down) {
                direction = if x_delta < 0 {
                    Direction::Right
                } else {
                    Direction::Left
                };
            }
            let new_coords =
                get_next_coords(&npc_coords, direction, self.file.width, self.file.height);

            if self.is_tile_walkable_npc(&new_coords) && !self.is_tile_occupied(&new_coords) {
                let npc = self.npcs.get_mut(&index).unwrap();
                npc.direction = direction;
                npc.coords = new_coords;
                npc.act_ticks = 0;
                Some(NpcUpdatePosition {
                    npc_index: index,
                    coords: npc.coords,
                    direction: npc.direction,
                })
            } else {
                let mut rng = rand::thread_rng();
                direction = Direction::from(rng.gen_range(0..=3));

                let new_coords =
                    get_next_coords(&npc_coords, direction, self.file.width, self.file.height);

                if self.is_tile_walkable_npc(&new_coords) && !self.is_tile_occupied(&new_coords) {
                    let npc = self.npcs.get_mut(&index).unwrap();
                    npc.direction = direction;
                    npc.coords = new_coords;
                    npc.act_ticks = 0;
                    Some(NpcUpdatePosition {
                        npc_index: index,
                        coords: npc.coords,
                        direction: npc.direction,
                    })
                } else {
                    None
                }
            }
        }
    }

    fn npc_get_chase_target_coords(&self, index: i32, npc_id: i32) -> Option<Coords> {
        match self.npc_get_chase_target_player_id(index, npc_id) {
            Some(player_id) => self
                .characters
                .get(&player_id)
                .map(|character| character.coords),
            None => None,
        }
    }

    // TODO: Party stuff
    fn npc_get_chase_target_player_id(&self, index: i32, npc_id: i32) -> Option<i32> {
        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return None,
        };

        let npc = match self.npcs.get(&index) {
            Some(npc) => npc,
            None => return None,
        };

        if !npc.opponents.is_empty() {
            let opponents_in_range = npc.opponents.iter().filter(|opponent| {
                let character = match self.characters.get(&opponent.player_id) {
                    Some(opponent) => opponent,
                    None => return false,
                };
                let distance = get_distance(&npc.coords, &character.coords);
                !character.hidden
                    && distance <= SETTINGS.npcs.chase_distance
                    && opponent.bored_ticks < SETTINGS.npcs.bored_timer
            });

            // get opponent with max damage dealt
            opponents_in_range
                .max_by(|a, b| a.damage_dealt.cmp(&b.damage_dealt))
                .map(|opponent| opponent.player_id)
        } else if npc_data.r#type == NpcType::Aggressive && !self.characters.is_empty() {
            // find closest player
            self.characters
                .iter()
                .filter(|(_, character)| {
                    let distance = get_distance(&npc.coords, &character.coords);
                    !character.hidden && distance <= SETTINGS.npcs.chase_distance
                })
                .min_by(|(_, a), (_, b)| {
                    let distance_a = get_distance(&npc.coords, &a.coords);
                    let distance_b = get_distance(&npc.coords, &b.coords);
                    distance_a.cmp(&distance_b)
                })
                .map(|(player_id, _)| *player_id)
        } else {
            None
        }
    }

    fn npc_get_attack_target_player_id(&self, index: i32) -> Option<i32> {
        let npc = match self.npcs.get(&index) {
            Some(npc) => npc,
            None => return None,
        };

        let adjacent_tiles = self.get_adjacent_tiles(&npc.coords);

        let adjacent_player_ids = self
            .characters
            .iter()
            .filter(|(_, character)| {
                adjacent_tiles
                    .iter()
                    .any(|coords| coords == &character.coords && !character.hidden)
            })
            .map(|(player_id, _)| *player_id)
            .collect::<Vec<_>>();

        let adjacent_opponent = npc
            .opponents
            .iter()
            .filter(|opponent| {
                adjacent_player_ids.contains(&opponent.player_id)
                    && opponent.bored_ticks < SETTINGS.npcs.bored_timer
            })
            .max_by_key(|opponent| opponent.damage_dealt);

        if let Some(opponent) = adjacent_opponent {
            Some(opponent.player_id)
        } else {
            let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return None,
            };

            // TODO: also attack adjacent players if blocking path to opponent(s)
            // Choose a random player if npc is aggressive
            if npc_data.r#type == NpcType::Aggressive {
                let mut rng = rand::thread_rng();
                adjacent_player_ids.choose(&mut rng).copied()
            } else {
                None
            }
        }
    }

    fn act_npc_move_idle(&mut self, index: i32) -> Option<NpcUpdatePosition> {
        let (direction, coords) = match self.npcs.get(&index) {
            Some(npc) => (npc.direction, npc.coords),
            None => return None,
        };
        // Logic ripped from EOServ..
        let mut rng = rand::thread_rng();
        let action = rng.gen_range(1..=10);

        if action == 10 {
            self.npcs.get_mut(&index).unwrap().walk_idle_for =
                Some(rng.gen_range(1..=4) * 1000 / SETTINGS.world.tick_rate);
            return None;
        }

        let new_direction = if (7..=9).contains(&action) {
            Direction::from(rng.gen_range(0..=3))
        } else {
            direction
        };

        let new_coords = get_next_coords(&coords, new_direction, self.file.width, self.file.height);

        if let Some(npc) = self.npcs.get_mut(&index) {
            npc.direction = new_direction;
            npc.act_ticks = 0;
            npc.walk_idle_for = None;
        }

        if self.is_tile_walkable_npc(&new_coords) && !self.is_tile_occupied(&new_coords) {
            if let Some(npc) = self.npcs.get_mut(&index) {
                npc.coords = new_coords;
            }

            Some(NpcUpdatePosition {
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
        index: i32,
        npc_id: i32,
        act_rate: i32,
        act_ticks: i32,
    ) -> Option<NpcUpdatePosition> {
        let (walk_idle_for, has_opponent) = {
            match self.npcs.get(&index) {
                Some(npc) => (npc.walk_idle_for.unwrap_or(0), !npc.opponents.is_empty()),
                None => return None,
            }
        };

        let idle_rate = act_rate + walk_idle_for;

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return None,
        };

        if npc_data.r#type == NpcType::Aggressive || has_opponent {
            self.act_npc_move_chase(index, npc_id, npc_data.r#type)
        } else if act_ticks >= idle_rate {
            self.act_npc_move_idle(index)
        } else {
            None
        }
    }

    fn act_npc_attack(&mut self, index: i32, npc_id: i32) -> Option<NpcUpdateAttack> {
        let target_player_id = match self.npc_get_attack_target_player_id(index) {
            Some(player_id) => player_id,
            None => return None,
        };

        let (damage, direction) = {
            let character = match self.characters.get(&target_player_id) {
                Some(character) => character,
                None => return None,
            };

            let npc = match self.npcs.get(&index) {
                Some(npc) => npc,
                None => return None,
            };

            let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return None,
            };

            let xdiff = npc.coords.x - character.coords.x;
            let ydiff = npc.coords.y - character.coords.y;

            let direction = match (xdiff, ydiff) {
                (0, 1) => Direction::Up,
                (0, -1) => Direction::Down,
                (1, 0) => Direction::Left,
                (-1, 0) => Direction::Right,
                (0, 0) => npc.direction,
                _ => return None,
            };

            (get_damage_amount(npc, npc_data, character), direction)
        };

        let (killed_state, hp_percentage) = {
            let character = match self.characters.get_mut(&target_player_id) {
                Some(character) => character,
                None => return None,
            };

            character.hp -= damage;

            let hp_percentage = character.get_hp_percentage();

            if damage > 0 {
                character
                    .player
                    .as_ref()
                    .unwrap()
                    .update_party_hp(hp_percentage);
            }

            let killed_state = if character.hp == 0 {
                PlayerKilledState::Killed
            } else {
                PlayerKilledState::Alive
            };

            (killed_state, hp_percentage)
        };

        if let Some(npc) = self.npcs.get_mut(&index) {
            npc.direction = direction;
            npc.act_ticks = 0;

            if killed_state == PlayerKilledState::Killed {
                npc.opponents
                    .retain(|opponent| opponent.player_id != target_player_id);
            }
        }

        Some(NpcUpdateAttack {
            npc_index: index,
            killed: killed_state,
            direction,
            player_id: target_player_id,
            damage,
            hp_percentage,
        })
    }

    fn act_npc(
        &mut self,
        index: i32,
    ) -> (
        Option<NpcUpdatePosition>,
        Option<NpcUpdateChat>,
        Option<NpcUpdateAttack>,
    ) {
        let (npc_id, spawn_index, act_ticks) = match self.npcs.get_mut(&index) {
            Some(npc) => {
                if !npc.alive {
                    return (None, None, None);
                } else {
                    for opponent in npc.opponents.iter_mut() {
                        opponent.bored_ticks += SETTINGS.npcs.act_rate;
                    }

                    npc.act_ticks += SETTINGS.npcs.act_rate;
                    npc.talk_ticks += SETTINGS.npcs.act_rate;
                    (npc.id, npc.spawn_index, npc.act_ticks)
                }
            }
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

        if act_rate == 0 || act_ticks == 0 || act_ticks < act_rate {
            (None, talk_update, None)
        } else {
            let attack_update = self.act_npc_attack(index, npc_id);
            let pos_update = if attack_update.is_some() {
                None
            } else {
                self.act_npc_move(index, npc_id, act_rate, act_ticks)
            };
            (pos_update, talk_update, attack_update)
        }
    }

    pub fn act_npcs(&mut self) {
        if self.npcs.is_empty() || SETTINGS.npcs.freeze_on_empty_map && self.characters.is_empty() {
            return;
        }

        if !self.npcs_initialized {
            self.npcs_initialized = true;
            for (spawn_index, spawn) in self.file.npcs.iter().enumerate() {
                let npcs = {
                    self.npcs
                        .iter()
                        .filter(|(_, npc)| npc.spawn_index == spawn_index && npc.id == spawn.id)
                        .map(|(index, _)| *index)
                        .collect::<Vec<i32>>()
                        .clone()
                };

                for (spawn_index, index) in npcs.into_iter().enumerate() {
                    let npc = self.npcs.get_mut(&index).unwrap();
                    npc.act_ticks = 0;
                    npc.talk_ticks = -60 * spawn_index as i32;
                }
            }
        }

        let mut attack_updates: Vec<NpcUpdateAttack> = Vec::with_capacity(self.npcs.len());
        let mut position_updates: Vec<NpcUpdatePosition> = Vec::with_capacity(self.npcs.len());
        let mut talk_updates: Vec<NpcUpdateChat> = Vec::with_capacity(self.npcs.len());

        let indexes = self.npcs.keys().cloned().collect::<Vec<i32>>();
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
            let characters = self.characters.keys().copied().collect::<Vec<_>>();

            for player_id in characters {
                let coords = match self.characters.get(&player_id) {
                    Some(character) => character.coords,
                    None => continue,
                };

                let in_range_npc_indexes: Vec<i32> = self
                    .npcs
                    .iter()
                    .filter(|(_, n)| in_range(&coords, &n.coords))
                    .map(|(i, _)| i)
                    .cloned()
                    .collect();

                let position_updates_in_rage: Vec<NpcUpdatePosition> = position_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                let mut talk_updates_in_range: Vec<NpcUpdateChat> = talk_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                let attack_updates_in_range: Vec<NpcUpdateAttack> = attack_updates
                    .iter()
                    .filter(|update| in_range_npc_indexes.contains(&update.npc_index))
                    .cloned()
                    .collect();

                let quest_talk_updates_in_rage: Vec<NpcUpdateChat> =
                    self.get_quest_talk_updates(&player_id, &in_range_npc_indexes);

                for talk_update in &quest_talk_updates_in_rage {
                    talk_updates_in_range.retain(|u| u.npc_index != talk_update.npc_index);
                    talk_updates_in_range.push(talk_update.clone());
                }

                if !position_updates_in_rage.is_empty()
                    || !talk_updates_in_range.is_empty()
                    || !attack_updates_in_range.is_empty()
                {
                    let packet = NpcPlayerServerPacket {
                        positions: position_updates_in_rage,
                        attacks: attack_updates_in_range,
                        chats: talk_updates_in_range,
                        hp: None,
                        tp: None,
                    };

                    let player = match self.characters.get(&player_id) {
                        Some(character) => match character.player {
                            Some(ref player) => player,
                            None => continue,
                        },
                        None => continue,
                    };

                    player.send(PacketAction::Player, PacketFamily::Npc, &packet);

                    let player_died = packet.attacks.iter().any(|update| {
                        update.player_id == player_id && update.killed == PlayerKilledState::Killed
                    });

                    if player_died {
                        player.die();
                    }
                }
            }
        }
    }

    fn get_quest_talk_updates(
        &mut self,
        player_id: &i32,
        npc_indexes: &[i32],
    ) -> Vec<NpcUpdateChat> {
        let quests_progress = match self.characters.get(player_id) {
            Some(character) => character.quests.clone(),
            None => return Vec::new(),
        };

        npc_indexes
            .iter()
            .filter_map(|index| {
                let npc = match self.npcs.get_mut(index) {
                    Some(npc) => npc,
                    None => return None,
                };

                if npc.talk_ticks < SETTINGS.npcs.talk_rate {
                    return None;
                }

                let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                    Some(npc_data) => npc_data,
                    None => return None,
                };

                if npc_data.r#type != NpcType::Quest {
                    return None;
                }

                npc.talk_ticks = 0;

                let mut possible_messages = Vec::new();

                for progress in &quests_progress {
                    let quest = match QUEST_DB.get(&progress.id) {
                        Some(quest) => quest,
                        None => continue,
                    };

                    let state = match quest.states.get(progress.state as usize) {
                        Some(state) => state,
                        None => continue,
                    };

                    let chat_action = match state.actions.iter().find(|action| {
                        action.name == "AddNpcChat"
                            && action.args[0] == Arg::Int(npc_data.behavior_id)
                    }) {
                        Some(chat_action) => chat_action,
                        None => continue,
                    };

                    let message = match chat_action.args.get(1) {
                        Some(Arg::Str(message)) => message.to_owned(),
                        _ => continue,
                    };

                    if !possible_messages.contains(&message) {
                        possible_messages.push(message);
                    }
                }

                for (_, quest) in QUEST_DB.iter() {
                    let state = match quest.states.first() {
                        Some(state) => state,
                        None => continue,
                    };

                    let chat_action = match state.actions.iter().find(|action| {
                        action.name == "AddNpcChat"
                            && action.args[0] == Arg::Int(npc_data.behavior_id)
                    }) {
                        Some(chat_action) => chat_action,
                        None => continue,
                    };

                    let message = match chat_action.args.get(1) {
                        Some(Arg::Str(message)) => message.to_owned(),
                        _ => continue,
                    };

                    if !possible_messages.contains(&message) {
                        possible_messages.push(message);
                    }
                }

                if !possible_messages.is_empty() {
                    let mut rng = rand::thread_rng();
                    Some(NpcUpdateChat {
                        npc_index: *index,
                        message: possible_messages.choose(&mut rng).unwrap().to_string(),
                    })
                } else {
                    None
                }
            })
            .collect::<Vec<_>>()
    }
}

fn get_damage_amount(npc: &Npc, npc_data: &EnfRecord, character: &Character) -> i32 {
    let mut rng = rand::thread_rng();
    let rand = rng.gen_range(0.0..=1.0);

    let amount = rng.gen_range(npc_data.min_damage..=npc_data.max_damage);

    let npc_facing_player_back_or_side =
        (i32::from(character.direction) - i32::from(npc.direction)).abs() != 2;

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
        Ok(amount) => cmp::min(amount.floor() as i32, character.hp),
        Err(e) => {
            error!("Failed to calculate damage: {}", e);
            0
        }
    }
}
