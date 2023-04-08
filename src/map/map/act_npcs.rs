use chrono::{Duration, Utc};
use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::npc, Coords, Direction, NPCUpdateAttack, NPCUpdateChat, NPCUpdatePos, PacketAction,
        PacketFamily,
    },
    pubs::{EnfNpc, EnfNpcType},
};
use rand::Rng;

use crate::{NPC_DB, SETTINGS, TALK_DB};

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

    fn act_npc_attack(
        &mut self,
        index: EOChar,
        npc_id: EOShort,
        npc_data: &EnfNpc,
    ) -> Option<NPCUpdateAttack> {
        None
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
        if act_delta < Duration::milliseconds(act_rate as i64) {
            (None, talk_update, None)
        } else {
            let pos_update = self.act_npc_move(index, npc_id, npc_data, act_rate, &act_delta);
            let attack_update = self.act_npc_attack(index, npc_id, npc_data);
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

                if !position_updates_in_rage.is_empty() || !talk_updates_in_range.is_empty() {
                    let packet = npc::Player {
                        pos: position_updates_in_rage,
                        attack: Vec::new(),
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
                }
            }
        }
    }
}
