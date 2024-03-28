use chrono::Utc;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{
                AttackPlayerServerPacket, CastAcceptServerPacket, CastReplyServerPacket,
                CastSpecServerPacket, LevelUpStats, NpcAcceptServerPacket, NpcJunkServerPacket,
                NpcKillStealProtectionState, NpcKilledData, NpcReplyServerPacket,
                NpcSpecServerPacket, PartyExpShare, RecoverReplyServerPacket,
                RecoverTargetGroupServerPacket,
            },
            PacketAction, PacketFamily,
        },
        Coords, Direction,
    },
};
use evalexpr::{context_map, eval_float_with_context};
use rand::Rng;

use crate::{map::Item, player::PlayerHandle, DROP_DB, FORMULAS, NPC_DB};

use super::super::Map;

struct ExpGain {
    pub player_id: i32,
    pub leveled_up: bool,
    pub level: i32,
    pub experience_gained: i32,
    pub total_experience: i32,
}

impl Map {
    pub fn attack_npc_reply(
        &mut self,
        player_id: i32,
        npc_index: i32,
        direction: Direction,
        damage_dealt: i32,
        spell_id: Option<i32>,
        protected: bool,
    ) {
        if spell_id.is_none() {
            let reply = AttackPlayerServerPacket {
                player_id,
                direction,
            };

            self.send_packet_near_player(
                player_id,
                PacketAction::Player,
                PacketFamily::Attack,
                &reply,
            );
        }

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        if let Some(spell_id) = spell_id {
            let mut packet = CastReplyServerPacket {
                spell_id,
                caster_id: player_id,
                caster_direction: direction,
                npc_index,
                damage: damage_dealt,
                hp_percentage: npc.get_hp_percentage(),
                caster_tp: match self.characters.get(&player_id) {
                    Some(character) => character.tp,
                    None => 0,
                },
                kill_steal_protection: Some(if protected {
                    NpcKillStealProtectionState::Protected
                } else {
                    NpcKillStealProtectionState::Unprotected
                }),
            };

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            character.player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::Cast,
                &packet,
            );

            packet.kill_steal_protection = None;

            self.send_packet_near_exclude_player(
                &npc.coords,
                player_id,
                PacketAction::Reply,
                PacketFamily::Cast,
                &packet,
            );
        } else {
            let mut packet = NpcReplyServerPacket {
                player_id,
                player_direction: direction,
                npc_index,
                damage: damage_dealt,
                hp_percentage: npc.get_hp_percentage(),
                kill_steal_protection: Some(if protected {
                    NpcKillStealProtectionState::Protected
                } else {
                    NpcKillStealProtectionState::Unprotected
                }),
            };

            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            character.player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::Npc,
                &packet,
            );

            packet.kill_steal_protection = None;

            self.send_packet_near_exclude_player(
                &npc.coords,
                player_id,
                PacketAction::Reply,
                PacketFamily::Npc,
                &packet,
            );
        }
    }

    pub async fn attack_npc_killed_reply(
        &mut self,
        killer_player_id: i32,
        npc_index: i32,
        damage_dealt: i32,
        spell_id: Option<i32>,
    ) {
        let (npc_id, npc_coords, is_boss) = match self.npcs.get(&npc_index) {
            Some(npc) => (npc.id, npc.coords, npc.boss),
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        let mut exp_gains: Vec<ExpGain> = Vec::new();

        let party = self.world.get_player_party(killer_player_id).await;

        if let Some(party) = party.as_ref() {
            let members_on_map: Vec<&i32> = party
                .members
                .iter()
                .filter(|id| self.characters.contains_key(id))
                .collect();

            let experience = if members_on_map.len() > 1 {
                let context = match context_map! {
                    "members" => members_on_map.len() as f64,
                    "exp" => npc_data.experience as f64,
                } {
                    Ok(context) => context,
                    Err(e) => {
                        error!("Failed to generate formula context: {}", e);
                        return;
                    }
                };

                match eval_float_with_context(&FORMULAS.party_exp_share, &context) {
                    Ok(experience) => experience as i32,
                    Err(e) => {
                        error!("Failed to calculate party experience share: {}", e);
                        1
                    }
                }
            } else {
                npc_data.experience
            };

            for member_id in members_on_map {
                let (leveled_up, level, total_experience, experience_gained) =
                    self.give_experience(*member_id, experience);
                exp_gains.push(ExpGain {
                    player_id: *member_id,
                    leveled_up,
                    level,
                    total_experience,
                    experience_gained,
                });
            }
        } else {
            let (leveled_up, level, total_experience, experience_gained) =
                self.give_experience(killer_player_id, npc_data.experience);
            exp_gains.push(ExpGain {
                player_id: killer_player_id,
                leveled_up,
                level,
                total_experience,
                experience_gained,
            });
        }

        let drop = get_drop(killer_player_id, npc_id, &npc_coords);

        let (drop_index, drop_item_id, drop_amount) = match drop {
            Some(drop) => {
                let index = self.get_next_item_index(1);
                let drop_item_id = drop.id;
                let drop_amount = drop.amount;
                self.items.insert(index, drop);
                (index, drop_item_id, drop_amount)
            }
            None => (0, 0, 0),
        };

        let killer = match self.characters.get(&killer_player_id) {
            Some(character) => character,
            None => return,
        };

        let npc_killed_data = NpcKilledData {
            killer_id: killer_player_id,
            killer_direction: killer.direction,
            npc_index,
            drop_index,
            drop_id: drop_item_id,
            drop_coords: npc_coords,
            drop_amount,
            damage: damage_dealt,
        };

        let caster_tp = killer.tp;

        for (player_id, character) in self.characters.iter() {
            let exp_gain = exp_gains.iter().find(|gain| gain.player_id == *player_id);
            let leveled_up = match exp_gain {
                Some(gain) => gain.leveled_up,
                None => false,
            };

            if let Some(spell_id) = spell_id {
                if leveled_up && exp_gains.len() == 1 {
                    self.attack_npc_killed_with_spell_level_up(
                        character.player.as_ref().unwrap(),
                        spell_id,
                        npc_killed_data.clone(),
                        caster_tp,
                        exp_gain.unwrap().total_experience,
                        LevelUpStats {
                            level: character.level,
                            stat_points: character.stat_points,
                            skill_points: character.skill_points,
                            max_hp: character.max_hp,
                            max_tp: character.max_tp,
                            max_sp: character.max_sp,
                        },
                    );
                } else {
                    self.attack_npc_killed_with_spell(
                        character.player.as_ref().unwrap(),
                        spell_id,
                        npc_killed_data.clone(),
                        caster_tp,
                        exp_gain.map(|exp_gain| exp_gain.total_experience),
                    );
                }
            } else if leveled_up && exp_gains.len() == 1 {
                self.attack_npc_killed_level_up(
                    character.player.as_ref().unwrap(),
                    npc_killed_data.clone(),
                    exp_gain.unwrap().total_experience,
                    LevelUpStats {
                        level: character.level,
                        stat_points: character.stat_points,
                        skill_points: character.skill_points,
                        max_hp: character.max_hp,
                        max_tp: character.max_tp,
                        max_sp: character.max_sp,
                    },
                );
            } else {
                self.attack_npc_killed(
                    character.player.as_ref().unwrap(),
                    npc_killed_data.clone(),
                    exp_gain.map(|exp_gain| exp_gain.total_experience),
                );
            }
        }

        if party.is_some() {
            self.attack_npc_killed_leveled_up_party_reply(&exp_gains);

            let level_up_gains: Vec<PartyExpShare> = exp_gains
                .iter()
                .map(|exp_gain| PartyExpShare {
                    player_id: exp_gain.player_id,
                    experience: exp_gain.experience_gained,
                    level_up: if exp_gain.leveled_up {
                        exp_gain.level
                    } else {
                        0
                    },
                })
                .filter(|gain| gain.level_up > 0)
                .collect();

            if !level_up_gains.is_empty() {
                self.world
                    .update_party_exp(killer_player_id, level_up_gains);
            }
        }

        if is_boss {
            self.npcs
                .iter_mut()
                .filter(|(_, n)| n.child)
                .for_each(|(_, child)| {
                    child.alive = false;
                    child.hp = 0;
                    child.opponents.clear();
                    child.dead_since = Utc::now();
                });

            if let Some((_, child_npc)) = self.npcs.iter().find(|(_, n)| n.child) {
                let packet = NpcJunkServerPacket {
                    npc_id: child_npc.id,
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize NpcJunkServerPacket packet: {}", e);
                    return;
                }

                let buf = writer.to_byte_array();

                self.characters.iter().for_each(|(_, character)| {
                    character.player.as_ref().unwrap().send_buf(
                        PacketAction::Junk,
                        PacketFamily::Npc,
                        buf.clone(),
                    );
                });
            }
        }

        for gain in &exp_gains {
            if let Some(character) = self.characters.get_mut(&gain.player_id) {
                character.killed_npc(npc_id);
            }
        }
    }

    fn attack_npc_killed_level_up(
        &self,
        player: &PlayerHandle,
        npc_killed_data: NpcKilledData,
        experience: i32,
        level_up: LevelUpStats,
    ) {
        player.send(
            PacketAction::Accept,
            PacketFamily::Npc,
            &NpcAcceptServerPacket {
                npc_killed_data,
                experience,
                level_up,
            },
        );
    }

    fn attack_npc_killed(
        &self,
        player: &PlayerHandle,
        npc_killed_data: NpcKilledData,
        experience: Option<i32>,
    ) {
        player.send(
            PacketAction::Spec,
            PacketFamily::Npc,
            &NpcSpecServerPacket {
                npc_killed_data,
                experience,
            },
        );
    }

    fn attack_npc_killed_with_spell_level_up(
        &self,
        player: &PlayerHandle,
        spell_id: i32,
        npc_killed_data: NpcKilledData,
        caster_tp: i32,
        experience: i32,
        level_up: LevelUpStats,
    ) {
        player.send(
            PacketAction::Accept,
            PacketFamily::Cast,
            &CastAcceptServerPacket {
                spell_id,
                npc_killed_data,
                caster_tp,
                experience,
                level_up,
            },
        );
    }

    fn attack_npc_killed_with_spell(
        &self,
        player: &PlayerHandle,
        spell_id: i32,
        npc_killed_data: NpcKilledData,
        caster_tp: i32,
        experience: Option<i32>,
    ) {
        player.send(
            PacketAction::Spec,
            PacketFamily::Cast,
            &CastSpecServerPacket {
                spell_id,
                npc_killed_data,
                caster_tp,
                experience,
            },
        );
    }

    fn attack_npc_killed_leveled_up_party_reply(&self, exp_gains: &Vec<ExpGain>) {
        for exp_gain in exp_gains {
            if exp_gain.leveled_up {
                let character = match self.characters.get(&exp_gain.player_id) {
                    Some(character) => character,
                    None => continue,
                };

                character.player.as_ref().unwrap().send(
                    PacketAction::TargetGroup,
                    PacketFamily::Recover,
                    &RecoverTargetGroupServerPacket {
                        stat_points: character.stat_points,
                        skill_points: character.skill_points,
                        max_hp: character.max_hp,
                        max_tp: character.max_tp,
                        max_sp: character.max_sp,
                    },
                );

                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Recover,
                    &RecoverReplyServerPacket {
                        experience: character.experience,
                        karma: character.karma,
                        level_up: Some(character.level),
                        stat_points: Some(character.stat_points),
                        skill_points: Some(character.skill_points),
                    },
                );
            }
        }
    }
}

fn get_drop(target_player_id: i32, npc_id: i32, npc_coords: &Coords) -> Option<Item> {
    if let Some(drop_npc) = DROP_DB.npcs.iter().find(|d| d.npc_id == npc_id) {
        let mut rng = rand::thread_rng();
        let mut drops = drop_npc.drops.clone();
        drops.sort_by(|a, b| a.rate.cmp(&b.rate));

        for drop in drops {
            let roll = rng.gen_range(0..=64000);
            if roll <= drop.rate {
                let amount = rng.gen_range(drop.min_amount..=drop.max_amount);
                return Some(Item {
                    id: drop.item_id,
                    amount,
                    coords: *npc_coords,
                    owner: target_player_id,
                });
            }
        }
    }

    None
}
