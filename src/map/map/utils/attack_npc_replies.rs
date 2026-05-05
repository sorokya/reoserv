use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        Coords, Direction,
        net::{
            PacketAction, PacketFamily,
            server::{
                AttackPlayerServerPacket, CastAcceptServerPacket, CastReplyServerPacket,
                CastSpecServerPacket, LevelUpStats, NpcAcceptServerPacket, NpcJunkServerPacket,
                NpcKillStealProtectionState, NpcKilledData, NpcReplyServerPacket,
                NpcSpecServerPacket, PartyExpShare, PartyTargetGroupServerPacket,
                RecoverReplyServerPacket, RecoverTargetGroupServerPacket,
            },
        },
    },
};
use evalexpr::{DefaultNumericTypes, HashMapContext, context_map, eval_float_with_context};
use rand::RngExt;

use crate::{
    DROP_DB, FORMULAS, GLOBAL_DROPS, NPC_DB, SETTINGS,
    deep::{BossPingServerPacket, FAMILY_BOSS},
    map::Item,
    utils::in_client_range,
};

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

        let npc = match self.npcs.iter().find(|npc| npc.index == npc_index) {
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
                caster_tp: self
                    .characters
                    .get(&player_id)
                    .map(|character| character.tp),
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

            if let Some(player) = character.player.as_ref() {
                player.send(PacketAction::Reply, PacketFamily::Cast, &packet);
            }

            packet.caster_tp = None;
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

            if let Some(player) = character.player.as_ref() {
                player.send(PacketAction::Reply, PacketFamily::Npc, &packet);
            }

            packet.kill_steal_protection = None;

            self.send_packet_near_exclude_player(
                &npc.coords,
                player_id,
                PacketAction::Reply,
                PacketFamily::Npc,
                &packet,
            );
        }

        if npc.boss {
            let packet = BossPingServerPacket {
                npc_index,
                npc_id: npc.id,
                hp: npc.hp,
                hp_percentage: npc.get_hp_percentage(),
                killed: false,
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                tracing::error!("Failed to serialize BossPingServerPacket: {}", e);
                return;
            }

            let buf = writer.to_byte_array();

            for player in self.characters.values().filter_map(|c| {
                if c.is_deep && in_client_range(&c.coords, &npc.coords) {
                    c.player.as_ref()
                } else {
                    None
                }
            }) {
                player.send_buf(
                    PacketAction::Ping,
                    PacketFamily::Unrecognized(FAMILY_BOSS),
                    buf.clone(),
                );
            }
        }
    }

    pub async fn attack_npc_killed_reply(
        &mut self,
        killer_player_id: i32,
        npc_index: i32,
        damage_dealt: i32,
        spell_id: Option<i32>,
    ) {
        let (npc_id, npc_coords, is_boss) =
            match self.npcs.iter().find(|npc| npc.index == npc_index) {
                Some(npc) => (npc.id, npc.coords, npc.boss),
                None => return,
            };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        let mut exp_gains: Vec<ExpGain> = Vec::new();

        let party = self
            .world
            .get_player_party(killer_player_id)
            .await
            .expect("Failed to get player party. Timeout");

        if let Some(party) = party.as_ref() {
            let members_on_map: Vec<&i32> = party
                .members
                .iter()
                .filter(|id| self.characters.contains_key(id))
                .collect();

            let experience = if members_on_map.len() > 1 {
                let context: HashMapContext<DefaultNumericTypes> = match context_map! {
                    "members" =>  float members_on_map.len() as f32,
                    "exp" => float npc_data.experience,
                } {
                    Ok(context) => context,
                    Err(e) => {
                        tracing::error!("Failed to generate formula context: {}", e);
                        return;
                    }
                };

                match eval_float_with_context(&FORMULAS.party_exp_share, &context) {
                    Ok(experience) => experience as i32,
                    Err(e) => {
                        tracing::error!("Failed to calculate party experience share: {}", e);
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
                match self.add_item(
                    drop.id,
                    drop.amount,
                    drop.coords,
                    drop.owner,
                    drop.protected_ticks,
                ) {
                    Ok(index) => (index, drop.id, drop.amount),
                    Err(e) => {
                        tracing::error!("Failed to add NPC drop to map: {}", e);
                        (0, 0, 0)
                    }
                }
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

        if let Some(party) = &party {
            self.attack_npc_killed_leveled_up_party_reply(&exp_gains, killer_player_id);

            let party_gains: Vec<PartyExpShare> = exp_gains
                .iter()
                .filter(|exp_gain| exp_gain.player_id != killer_player_id)
                .map(|exp_gain| PartyExpShare {
                    player_id: exp_gain.player_id,
                    experience: exp_gain.experience_gained,
                    level_up: if exp_gain.leveled_up {
                        exp_gain.level
                    } else {
                        0
                    },
                })
                .collect();

            if !party_gains.is_empty() {
                for (player_id, character) in self.characters.iter() {
                    if *player_id == killer_player_id || party.members.contains(player_id) {
                        continue;
                    }

                    let player = match character.player.as_ref() {
                        Some(player) => player,
                        None => continue,
                    };

                    let gains = party_gains
                        .iter()
                        .filter_map(|gain| {
                            let gain_character = match self.characters.get(&gain.player_id) {
                                Some(c) => c,
                                None => return None,
                            };

                            if gain.level_up > 0
                                && in_client_range(&character.coords, &gain_character.coords)
                            {
                                Some(gain.to_owned())
                            } else {
                                None
                            }
                        })
                        .collect::<Vec<_>>();

                    if !gains.is_empty() {
                        player.send(
                            PacketAction::TargetGroup,
                            PacketFamily::Party,
                            &PartyTargetGroupServerPacket { gains },
                        )
                    }
                }

                self.world.update_party_exp(killer_player_id, party_gains);
            }
        }

        let leveled_up = exp_gains
            .iter()
            .any(|gain| gain.player_id == killer_player_id && gain.leveled_up);
        for (player_id, character) in self.characters.iter() {
            let exp_gain = exp_gains.iter().find(|gain| gain.player_id == *player_id);

            if let Some(spell_id) = spell_id {
                if leveled_up {
                    if let Some(player) = character.player.as_ref() {
                        player.send(
                            PacketAction::Accept,
                            PacketFamily::Cast,
                            &CastAcceptServerPacket {
                                spell_id,
                                npc_killed_data: npc_killed_data.clone(),
                                caster_tp: if *player_id == killer_player_id {
                                    Some(character.tp)
                                } else {
                                    None
                                },
                                experience: if *player_id == killer_player_id {
                                    Some(character.experience)
                                } else {
                                    None
                                },
                                level_up: if *player_id == killer_player_id {
                                    Some(LevelUpStats {
                                        level: character.level,
                                        stat_points: character.stat_points,
                                        skill_points: character.skill_points,
                                        max_hp: character.max_hp,
                                        max_tp: character.max_tp,
                                        max_sp: character.max_sp,
                                    })
                                } else {
                                    None
                                },
                            },
                        );
                    }
                } else if let Some(player) = character.player.as_ref() {
                    player.send(
                        PacketAction::Spec,
                        PacketFamily::Cast,
                        &CastSpecServerPacket {
                            spell_id,
                            npc_killed_data: npc_killed_data.clone(),
                            caster_tp: if *player_id == killer_player_id {
                                Some(character.tp)
                            } else {
                                None
                            },
                            experience: if *player_id == killer_player_id {
                                exp_gain.map(|exp_gain| exp_gain.total_experience)
                            } else {
                                None
                            },
                        },
                    );
                }
            } else if leveled_up {
                if let Some(player) = character.player.as_ref() {
                    player.send(
                        PacketAction::Accept,
                        PacketFamily::Npc,
                        &NpcAcceptServerPacket {
                            npc_killed_data: npc_killed_data.clone(),
                            experience: if *player_id == killer_player_id {
                                Some(character.experience)
                            } else {
                                None
                            },
                            level_up: if *player_id == killer_player_id {
                                Some(LevelUpStats {
                                    level: character.level,
                                    stat_points: character.stat_points,
                                    skill_points: character.skill_points,
                                    max_hp: character.max_hp,
                                    max_tp: character.max_tp,
                                    max_sp: character.max_sp,
                                })
                            } else {
                                None
                            },
                        },
                    );
                }
            } else if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Spec,
                    PacketFamily::Npc,
                    &NpcSpecServerPacket {
                        npc_killed_data: npc_killed_data.clone(),
                        experience: if *player_id == killer_player_id {
                            Some(character.experience)
                        } else {
                            None
                        },
                    },
                );
            }
        }

        if is_boss {
            self.npcs
                .iter_mut()
                .filter(|npc| npc.child)
                .for_each(|child| {
                    child.alive = false;
                    child.hp = 0;
                    child.opponents.clear();

                    if child.spawn_index.is_some() {
                        child.spawn_ticks = child.spawn_time;
                    }
                });

            if let Some(child_npc) = self.npcs.iter().find(|npc| npc.child) {
                self.send_packet_all(
                    PacketAction::Junk,
                    PacketFamily::Npc,
                    NpcJunkServerPacket {
                        npc_id: child_npc.id,
                    },
                );
            }

            let packet = BossPingServerPacket {
                npc_index,
                npc_id,
                hp: 0,
                hp_percentage: 0,
                killed: true,
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                tracing::error!("Failed to serialize BossPingServerPacket: {}", e);
                return;
            }

            let buf = writer.to_byte_array();

            for player in self.characters.values().filter_map(|c| {
                if c.is_deep && in_client_range(&c.coords, &npc_coords) {
                    c.player.as_ref()
                } else {
                    None
                }
            }) {
                player.send_buf(
                    PacketAction::Ping,
                    PacketFamily::Unrecognized(FAMILY_BOSS),
                    buf.clone(),
                );
            }
        }

        for gain in &exp_gains {
            if let Some(character) = self.characters.get_mut(&gain.player_id) {
                character.killed_npc(npc_id);
            }
        }
    }

    fn attack_npc_killed_leveled_up_party_reply(
        &self,
        exp_gains: &Vec<ExpGain>,
        killer_player_id: i32,
    ) {
        for exp_gain in exp_gains {
            if exp_gain.leveled_up {
                let character = match self.characters.get(&exp_gain.player_id) {
                    Some(character) => character,
                    None => continue,
                };

                let player = match character.player.as_ref() {
                    Some(player) => player,
                    None => continue,
                };

                if exp_gain.player_id == killer_player_id {
                    continue;
                }

                player.send(
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

                player.send(
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
    let mut drops = GLOBAL_DROPS.drops.iter().collect::<Vec<_>>();
    if let Some(drop_npc) = DROP_DB.npcs.iter().find(|d| d.npc_id == npc_id) {
        drops.extend(drop_npc.drops.iter());
    }

    if !drops.is_empty() {
        let mut rng = rand::rng();
        drops.sort_by_key(|a| a.rate);

        for drop in drops {
            let roll = rng.random_range(0..=64000);
            if roll <= drop.rate {
                let amount = rng.random_range(drop.min_amount..=drop.max_amount);
                if amount > 0 {
                    return Some(Item {
                        index: -1,
                        id: drop.item_id,
                        amount,
                        coords: *npc_coords,
                        owner: target_player_id,
                        protected_ticks: SETTINGS.load().world.drop_protect_npc,
                    });
                }
            }
        }
    }

    None
}
