use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::MapTimedEffect,
        net::{
            server::{
                EffectSpecServerPacket, EffectSpecServerPacketMapDamageTypeData,
                EffectSpecServerPacketMapDamageTypeDataTpDrain, EffectTargetOtherServerPacket,
                MapDamageType, MapDrainDamageOther,
            },
            PacketAction, PacketFamily,
        },
    },
};

use crate::{utils::in_client_range, SETTINGS};

use super::super::Map;

impl Map {
    pub fn timed_drain(&mut self) {
        if self.file.timed_effect == MapTimedEffect::HpDrain {
            self.timed_drain_hp();
        }

        if self.file.timed_effect == MapTimedEffect::TpDrain {
            self.timed_drain_tp();
        }
    }

    fn timed_drain_hp(&mut self) {
        let player_ids: Vec<i32> = self.characters.keys().copied().collect();
        let mut damage_list: Vec<i32> = Vec::with_capacity(player_ids.len());

        for player_id in &player_ids {
            let character = match self.characters.get_mut(player_id) {
                Some(character) => character,
                None => {
                    damage_list.push(0);
                    continue;
                }
            };

            if character.hidden {
                damage_list.push(0);
                continue;
            }

            let damage = (character.max_hp as f32 * SETTINGS.world.drain_hp_damage).floor() as i32;
            let damage = cmp::min(damage, character.hp - 1);
            let damage = cmp::max(damage, 0) as i32;

            character.hp -= damage;
            damage_list.push(damage);
        }

        for (index, player_id) in player_ids.iter().enumerate() {
            let damage = match damage_list.get(index) {
                Some(damage) => *damage,
                None => 0,
            };

            let character = match self.characters.get(player_id) {
                Some(character) => character,
                None => continue,
            };

            if damage > 0 {
                character
                    .player
                    .as_ref()
                    .unwrap()
                    .update_party_hp(character.get_hp_percentage());
            }

            let packet = EffectTargetOtherServerPacket {
                damage,
                hp: character.hp,
                max_hp: character.max_hp,
                others: player_ids
                    .iter()
                    .enumerate()
                    .filter_map(|(other_index, id)| {
                        if id == player_id {
                            None
                        } else {
                            match self.characters.get(id) {
                                Some(other) => {
                                    if other.hidden
                                        || !in_client_range(&character.coords, &other.coords)
                                    {
                                        None
                                    } else {
                                        let other_damage = match damage_list.get(other_index) {
                                            Some(damage) => *damage,
                                            None => 0,
                                        };
                                        if other_damage > 0 {
                                            Some(MapDrainDamageOther {
                                                player_id: *id,
                                                hp_percentage: other.get_hp_percentage(),
                                                damage: other_damage,
                                            })
                                        } else {
                                            None
                                        }
                                    }
                                }
                                None => None,
                            }
                        }
                    })
                    .collect(),
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize EffectTargetOtherServerPacket: {}", e);
                return;
            }

            character.player.as_ref().unwrap().send(
                PacketAction::TargetOther,
                PacketFamily::Effect,
                writer.to_byte_array(),
            );
        }
    }

    fn timed_drain_tp(&mut self) {
        for character in self.characters.values_mut() {
            if character.tp == 0 || character.hidden {
                continue;
            }

            let damage = (character.max_tp as f32 * SETTINGS.world.drain_tp_damage).floor() as i32;
            let damage = cmp::min(damage, character.tp - 1);
            let damage = cmp::max(damage, 0) as i32;

            character.tp -= damage;

            let packet = EffectSpecServerPacket {
                map_damage_type: MapDamageType::TpDrain,
                map_damage_type_data: Some(EffectSpecServerPacketMapDamageTypeData::TpDrain(
                    EffectSpecServerPacketMapDamageTypeDataTpDrain {
                        tp_damage: damage,
                        tp: character.tp,
                        max_tp: character.max_tp,
                    },
                )),
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize EffectSpecServerPacket: {}", e);
                return;
            }

            character.player.as_ref().unwrap().send(
                PacketAction::Spec,
                PacketFamily::Effect,
                writer.to_byte_array(),
            );
        }
    }
}
