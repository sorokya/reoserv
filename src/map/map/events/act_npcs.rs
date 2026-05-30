use std::{cmp, time::Duration};

use eolib::protocol::{
    Coords, Direction,
    net::{
        PacketAction, PacketFamily,
        server::{
            NpcPlayerServerPacket, NpcUpdateAttack, NpcUpdateChat, NpcUpdatePosition,
            PlayerKilledState, SitState,
        },
    },
    r#pub::{EnfRecord, NpcType},
};

use evalexpr::{DefaultNumericTypes, HashMapContext, context_map, eval_float_with_context};
use rand::{RngExt, seq::IndexedRandom};

use crate::{
    FORMULAS, NPC_DB, SETTINGS, TALK_DB,
    character::Character,
    map::Npc,
    utils::{get_distance, get_next_coords, in_range},
};

use super::super::Map;

const STATIONARY_SPAWN_TYPE: i32 = 7;
const FLUSH_INTERVAL_MS: u64 = 50;

impl Map {
    fn act_npc_talk(&mut self, index: i32, npc_id: i32) -> Option<NpcUpdateChat> {
        let talk_db = TALK_DB.load();
        let talk_record = talk_db.npcs.iter().find(|record| record.npc_id == npc_id)?;

        let npc = self.npcs.iter_mut().find(|npc| npc.index == index)?;

        if !npc.alive || npc.talk_ticks < SETTINGS.load().npcs.talk_rate {
            return None;
        }

        npc.talk_ticks = 0;

        let mut rng = rand::rng();
        let roll = rng.random_range(0..=100);
        if roll <= talk_record.rate {
            let message_index = rng.random_range(0..talk_record.messages.len());
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

        let npc_coords = self.npcs.iter().find_map(|npc| {
            if npc.index == index {
                Some(npc.coords)
            } else {
                None
            }
        })?;

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
            let npc = self.npcs.iter_mut().find(|npc| npc.index == index)?;
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
                let npc = self.npcs.iter_mut().find(|npc| npc.index == index)?;
                npc.direction = direction;
                npc.coords = new_coords;
                npc.act_ticks = 0;
                Some(NpcUpdatePosition {
                    npc_index: index,
                    coords: npc.coords,
                    direction: npc.direction,
                })
            } else {
                let mut rng = rand::rng();
                direction = Direction::from(rng.random_range(0..=3));

                let new_coords =
                    get_next_coords(&npc_coords, direction, self.file.width, self.file.height);

                if self.is_tile_walkable_npc(&new_coords) && !self.is_tile_occupied(&new_coords) {
                    let npc = self.npcs.iter_mut().find(|npc| npc.index == index)?;
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
        let npc_db = NPC_DB.load();
        let npc_data = npc_db.npcs.get(npc_id as usize - 1)?;

        let npc = self.npcs.iter().find(|npc| npc.index == index)?;

        if !npc.opponents.is_empty() {
            let opponents_in_range = npc.opponents.iter().filter(|opponent| {
                let character = match self.characters.get(&opponent.player_id) {
                    Some(opponent) => opponent,
                    None => return false,
                };
                let distance = get_distance(&npc.coords, &character.coords);
                !character.hidden
                    && !character.captcha_open
                    && distance <= SETTINGS.load().npcs.chase_distance
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
                    !character.hidden
                        && !character.captcha_open
                        && distance <= SETTINGS.load().npcs.chase_distance
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
        let npc = self.npcs.iter().find(|npc| npc.index == index)?;

        let adjacent_tiles = self.get_adjacent_tiles(&npc.coords);

        let adjacent_player_ids = self
            .characters
            .iter()
            .filter(|(_, character)| {
                adjacent_tiles.iter().any(|coords| {
                    coords == &character.coords && !character.hidden && !character.captcha_open
                })
            })
            .map(|(player_id, _)| *player_id)
            .collect::<Vec<_>>();

        let adjacent_opponent = npc
            .opponents
            .iter()
            .filter(|opponent| adjacent_player_ids.contains(&opponent.player_id))
            .max_by_key(|opponent| opponent.damage_dealt);

        if let Some(opponent) = adjacent_opponent {
            Some(opponent.player_id)
        } else {
            let npc_db = NPC_DB.load();
            let npc_data = npc_db.npcs.get(npc.id as usize - 1)?;

            // TODO: also attack adjacent players if blocking path to opponent(s)
            // Choose a random player if npc is aggressive
            if npc_data.r#type == NpcType::Aggressive {
                let mut rng = rand::rng();
                adjacent_player_ids.choose(&mut rng).copied()
            } else {
                None
            }
        }
    }

    fn act_npc_move_idle(&mut self, index: i32) -> Option<NpcUpdatePosition> {
        let (direction, coords) = self.npcs.iter().find_map(|npc| {
            if npc.index == index {
                Some((npc.direction, npc.coords))
            } else {
                None
            }
        })?;

        // Logic ripped from EOServ..
        let mut rng = rand::rng();
        let action = rng.random_range(1..=10);

        if action == 10 {
            self.npcs
                .iter_mut()
                .find(|npc| npc.index == index)?
                .walk_idle_for =
                Some(rng.random_range(1..=4) * 1000 / SETTINGS.load().world.tick_rate);
            return None;
        }

        let new_direction = if (7..=9).contains(&action) {
            Direction::from(rng.random_range(0..=3))
        } else {
            direction
        };

        let new_coords = get_next_coords(&coords, new_direction, self.file.width, self.file.height);

        if let Some(npc) = self.npcs.iter_mut().find(|npc| npc.index == index) {
            npc.direction = new_direction;
            npc.act_ticks = 0;
            npc.walk_idle_for = None;
        }

        if self.is_tile_walkable_npc(&new_coords) && !self.is_tile_occupied(&new_coords) {
            if let Some(npc) = self.npcs.iter_mut().find(|npc| npc.index == index) {
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

    fn act_npc_move(&mut self, index: i32, npc_id: i32) -> Option<NpcUpdatePosition> {
        let has_opponent = match self.npcs.iter().find(|npc| npc.index == index) {
            Some(npc) => !npc.opponents.is_empty(),
            None => return None,
        };

        let npc_db = NPC_DB.load();
        let npc_data = npc_db.npcs.get(npc_id as usize - 1)?;

        if npc_data.r#type == NpcType::Aggressive || has_opponent {
            self.act_npc_move_chase(index, npc_id, npc_data.r#type)
        } else {
            self.act_npc_move_idle(index)
        }
    }

    fn act_npc_attack(&mut self, index: i32, npc_id: i32) -> Option<NpcUpdateAttack> {
        let target_player_id = self.npc_get_attack_target_player_id(index)?;

        let (damage, direction) = {
            let character = self.characters.get(&target_player_id)?;

            let npc = self.npcs.iter().find(|npc| npc.index == index)?;

            let npc_db = NPC_DB.load();
            let npc_data = npc_db.npcs.get(npc_id as usize - 1)?;

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
            let character = self.characters.get_mut(&target_player_id)?;

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

        if let Some(npc) = self.npcs.iter_mut().find(|npc| npc.index == index) {
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
        let (npc_id, spawn_type) = match self.npcs.iter().find(|npc| npc.index == index) {
            Some(npc) if npc.alive => (npc.id, npc.spawn_type),
            _ => return (None, None, None),
        };

        let elapsed_ticks = if spawn_type == STATIONARY_SPAWN_TYPE {
            SETTINGS.load().npcs.talk_rate
        } else {
            act_rate_for_spawn_type(spawn_type)
        };

        if let Some(npc) = self.npcs.iter_mut().find(|npc| npc.index == index) {
            npc.talk_ticks += elapsed_ticks;
            for opponent in npc.opponents.iter_mut() {
                opponent.bored_ticks += elapsed_ticks;
            }
        }

        let talk_update = self.act_npc_talk(index, npc_id);

        if spawn_type == STATIONARY_SPAWN_TYPE {
            return (None, talk_update, None);
        }

        self.drop_opponents(index);
        let attack_update = self.act_npc_attack(index, npc_id);
        let pos_update = if attack_update.is_some() {
            None
        } else {
            self.act_npc_move(index, npc_id)
        };
        (pos_update, talk_update, attack_update)
    }

    fn drop_opponents(&mut self, index: i32) {
        let npc = match self.npcs.iter_mut().find(|npc| npc.index == index) {
            Some(npc) => npc,
            None => return,
        };

        npc.opponents
            .retain(|o| o.bored_ticks < SETTINGS.load().npcs.bored_timer);
    }

    pub fn npc_act(&mut self, index: i32, generation: u64) {
        let current_gen = match self.npcs.iter().find(|npc| npc.index == index) {
            Some(npc) if npc.alive => npc.act_gen,
            _ => return,
        };
        // Stale wake from a chain that's been superseded by reload/respawn.
        if current_gen != generation {
            return;
        }

        // When the map is frozen we let the chain die rather than spinning
        // a no-op tokio task per NPC per cadence. enter() re-bootstraps the
        // alive NPCs when the first character returns.
        let frozen =
            SETTINGS.load().npcs.freeze_on_empty_map && self.characters.is_empty();
        if frozen {
            return;
        }

        let (pos_update, chat_update, attack_update) = self.act_npc(index);
        // Snapshot post-act coords so a fast NPC that walks out of a player's
        // range during the 50ms flush window doesn't get its event silently
        // dropped from that player's batch.
        let npc_coords = match self.npcs.iter().find(|npc| npc.index == index) {
            Some(npc) => npc.coords,
            None => return,
        };
        self.enqueue_npc_act_updates(npc_coords, pos_update, chat_update, attack_update);
        self.schedule_next_npc_act(index);
    }

    fn enqueue_npc_act_updates(
        &mut self,
        coords: Coords,
        pos_update: Option<NpcUpdatePosition>,
        chat_update: Option<NpcUpdateChat>,
        attack_update: Option<NpcUpdateAttack>,
    ) {
        let had_any = pos_update.is_some() || chat_update.is_some() || attack_update.is_some();
        if let Some(p) = pos_update {
            self.pending_position_updates.push((coords, p));
        }
        if let Some(c) = chat_update {
            self.pending_chat_updates.push((coords, c));
        }
        if let Some(a) = attack_update {
            self.pending_attack_updates.push((coords, a));
        }
        if had_any && !self.flush_scheduled {
            self.flush_scheduled = true;
            self.schedule_next_flush();
        }
    }

    pub fn flush_npc_updates(&mut self) {
        self.flush_scheduled = false;

        if self.pending_position_updates.is_empty()
            && self.pending_attack_updates.is_empty()
            && self.pending_chat_updates.is_empty()
        {
            return;
        }

        let positions = std::mem::take(&mut self.pending_position_updates);
        let attacks = std::mem::take(&mut self.pending_attack_updates);
        let chats = std::mem::take(&mut self.pending_chat_updates);

        let player_ids = self.characters.keys().copied().collect::<Vec<_>>();
        for player_id in player_ids {
            let coords = match self.characters.get(&player_id) {
                Some(c) => c.coords,
                None => continue,
            };

            let positions_in_range: Vec<NpcUpdatePosition> = positions
                .iter()
                .filter(|(npc_coords, _)| in_range(&coords, npc_coords))
                .map(|(_, u)| u.clone())
                .collect();
            let attacks_in_range: Vec<NpcUpdateAttack> = attacks
                .iter()
                .filter(|(npc_coords, _)| in_range(&coords, npc_coords))
                .map(|(_, u)| u.clone())
                .collect();
            let chats_in_range: Vec<NpcUpdateChat> = chats
                .iter()
                .filter(|(npc_coords, _)| in_range(&coords, npc_coords))
                .map(|(_, u)| u.clone())
                .collect();

            if positions_in_range.is_empty()
                && attacks_in_range.is_empty()
                && chats_in_range.is_empty()
            {
                continue;
            }

            let packet = NpcPlayerServerPacket {
                positions: positions_in_range,
                attacks: attacks_in_range,
                chats: chats_in_range,
                hp: None,
                tp: None,
            };

            let player = match self.characters.get(&player_id) {
                Some(c) => match c.player.as_ref() {
                    Some(p) => p,
                    None => continue,
                },
                None => continue,
            };

            player.send(PacketAction::Player, PacketFamily::Npc, &packet);

            let died = packet.attacks.iter().any(|u| {
                u.player_id == player_id && u.killed == PlayerKilledState::Killed
            });
            if died {
                player.die();
            }
        }
    }

    pub fn schedule_next_flush(&self) {
        let handle = match self.self_handle.as_ref() {
            Some(h) => h.clone(),
            None => return,
        };
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(FLUSH_INTERVAL_MS)).await;
            handle.flush_npc_updates();
        });
    }

    pub fn bootstrap_npc_act(&mut self, index: i32) {
        match self.npcs.iter_mut().find(|npc| npc.index == index) {
            Some(npc) if npc.alive => {
                // Invalidate any orphan task from a previous chain.
                npc.act_gen = npc.act_gen.wrapping_add(1);
            }
            _ => return,
        }
        self.schedule_next_npc_act_with(index, true);
    }

    fn schedule_next_npc_act(&mut self, index: i32) {
        self.schedule_next_npc_act_with(index, false);
    }

    fn schedule_next_npc_act_with(&mut self, index: i32, initial: bool) {
        // Take walk_idle_for so the extra delay is consumed exactly once;
        // it's only set when act_npc_move_idle picks "stand still" and should
        // not keep adding to subsequent chase/attack reschedules.
        let (spawn_type, walk_idle_for_ticks, generation) =
            match self.npcs.iter_mut().find(|npc| npc.index == index) {
                Some(npc) if npc.alive => {
                    let extra = npc.walk_idle_for.take().unwrap_or(0);
                    (npc.spawn_type, extra, npc.act_gen)
                }
                _ => return,
            };

        let settings = SETTINGS.load();
        let tick_rate_ms = settings.world.tick_rate.max(1) as i64;

        let cadence_ticks = if spawn_type == STATIONARY_SPAWN_TYPE {
            settings.npcs.talk_rate.max(1) as i64
        } else {
            act_rate_for_spawn_type(spawn_type).max(1) as i64
        };

        let base_ms = (cadence_ticks * tick_rate_ms).max(1) as u64;
        let extra_ms = (walk_idle_for_ticks.max(0) as i64 * tick_rate_ms) as u64;

        let mut rng = rand::rng();
        let delay = if initial {
            // First wake after spawn: at least base/4 ms so a fresh mob can't
            // hit on frame 0, but spread up to base+extra so a spawn group
            // desyncs immediately.
            let span = base_ms.saturating_add(extra_ms).max(4);
            let floor = (base_ms / 4).max(1);
            rng.random_range(floor..span.max(floor + 1))
        } else {
            let jitter_range = (base_ms / 4).max(1);
            let jitter = rng.random_range(0..jitter_range);
            base_ms.saturating_add(extra_ms).saturating_add(jitter)
        };

        let handle = match self.self_handle.as_ref() {
            Some(h) => h.clone(),
            None => return, // map is shutting down
        };
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(delay)).await;
            handle.npc_act(index, generation);
        });
    }

}

fn act_rate_for_spawn_type(spawn_type: i32) -> i32 {
    let s = SETTINGS.load();
    match spawn_type {
        0 => s.npcs.speed_0,
        1 => s.npcs.speed_1,
        2 => s.npcs.speed_2,
        3 => s.npcs.speed_3,
        4 => s.npcs.speed_4,
        5 => s.npcs.speed_5,
        6 => s.npcs.speed_6,
        _ => s.npcs.speed_0,
    }
}

fn get_damage_amount(npc: &Npc, npc_data: &EnfRecord, character: &Character) -> i32 {
    let formulas = FORMULAS.load();
    let mut rng = rand::rng();
    let rand = rng.random_range(0.0..=1.0);

    // Some pub files have min_damage > max_damage; clamp instead of panicking.
    let min_damage = npc_data.min_damage;
    let max_damage = npc_data.max_damage.max(min_damage);
    let amount = rng.random_range(min_damage..=max_damage);

    let npc_facing_player_back_or_side =
        (i32::from(character.direction) - i32::from(npc.direction)).abs() != 2;

    let context: HashMapContext<DefaultNumericTypes> = match context_map! {
        "critical" => npc_facing_player_back_or_side,
        "damage" => float amount,
        "target_armor" => float character.armor,
        "target_sitting" => character.sit_state != SitState::Stand,
        "accuracy" => float npc_data.accuracy,
        "target_evade" => float character.evasion,
    } {
        Ok(context) => context,
        Err(e) => {
            tracing::error!("Failed to generate formula context: {}", e);
            return 0;
        }
    };

    let hit_rate = match eval_float_with_context(&formulas.hit_rate, &context) {
        Ok(hit_rate) => hit_rate,
        Err(e) => {
            tracing::error!("Failed to calculate hit rate: {}", e);
            0.0
        }
    };

    if hit_rate < rand {
        return 0;
    }

    match eval_float_with_context(&formulas.damage, &context) {
        Ok(amount) => cmp::min(amount.floor() as i32, character.hp),
        Err(e) => {
            tracing::error!("Failed to calculate damage: {}", e);
            0
        }
    }
}
