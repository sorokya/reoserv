use eolib::{protocol::{net::{server::{ItemReplyServerPacket, ItemReplyServerPacketItemTypeData, ItemReplyServerPacketItemTypeDataHeal, RecoverAgreeServerPacket, WarpEffect, ItemReplyServerPacketItemTypeDataEffectPotion, ItemReplyServerPacketItemTypeDataHairDye, AvatarAgreeServerPacket, AvatarChange, AvatarChangeType, AvatarChangeChangeTypeData, AvatarChangeChangeTypeDataHairColor, ItemReplyServerPacketItemTypeDataCureCurse, AvatarChangeChangeTypeDataEquipment}, PacketAction, PacketFamily, Item}, r#pub::{ItemType, ItemSpecial}, Coords}, data::{EoWriter, EoSerialize}};

use crate::{character::PaperdollSlot, ITEM_DB};

use super::super::Map;

impl Map {
    pub async fn use_item(&mut self, player_id: i32, item_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                return;
            }
        };

        if character.player.as_ref().unwrap().is_trading().await {
            return;
        }

        if !character.items.iter().any(|item| item.id == item_id) {
            return;
        }

        let item = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => {
                return;
            }
        };

        let mut reply = ItemReplyServerPacket::default();
        let player = character.player.as_ref().unwrap().clone();

        match item.r#type {
            ItemType::Heal => {
                let hp_gain = character.heal(item.hp);
                let tp_gain = character.tp_heal(item.tp);
                if hp_gain == 0 && tp_gain == 0 {
                    return;
                }
                reply.item_type_data = Some(ItemReplyServerPacketItemTypeData::Heal(ItemReplyServerPacketItemTypeDataHeal {
                    hp_gain,
                    hp: character.hp,
                    tp: character.tp,
                }));
                reply.item_type = ItemType::Heal;

                if hp_gain > 0 {
                    let packet = RecoverAgreeServerPacket {
                        player_id,
                        heal_hp: hp_gain,
                        hp_percentage: character.get_hp_percentage(),
                    };

                    player.update_party_hp(character.get_hp_percentage());

                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Agree,
                        PacketFamily::Recover,
                        packet,
                    );
                }
            }
            ItemType::Teleport => {
                if self.file.can_scroll {
                    return;
                }

                player.request_warp(
                    item.spec1,
                    Coords {
                        x: item.spec2,
                        y: item.spec3,
                    },
                    character.map_id == item.spec1,
                    Some(WarpEffect::Scroll),
                );
                reply.item_type = ItemType::Teleport;
            }
            ItemType::Alcohol => {
                reply.item_type = ItemType::Alcohol;
            }
            ItemType::EffectPotion => {
                reply.item_type = ItemType::EffectPotion;
                reply.item_type_data = Some(ItemReplyServerPacketItemTypeData::EffectPotion(ItemReplyServerPacketItemTypeDataEffectPotion {
                    effect_id: item.spec1,
                }));
                self.play_effect(player_id, item.spec1);
            }
            ItemType::HairDye => {
                reply.item_type = ItemType::HairDye;
                reply.item_type_data = Some(ItemReplyServerPacketItemTypeData::HairDye(ItemReplyServerPacketItemTypeDataHairDye {
                    hair_color: item.spec1,
                }));
                character.hair_color = item.spec1;
                let packet = AvatarAgreeServerPacket {
                    change: AvatarChange {
                        player_id,
                        change_type: AvatarChangeType::HairColor,
                        sound: false,
                        change_type_data: Some(AvatarChangeChangeTypeData::HairColor(AvatarChangeChangeTypeDataHairColor {
                            hair_color: item.spec1,
                        })),
                    },
                };
                self.send_packet_near_player(
                    player_id,
                    PacketAction::Agree,
                    PacketFamily::Avatar,
                    packet,
                );
            }
            ItemType::CureCurse => {
                let paperdoll = character.get_paperdoll_array();
                let mut cursed_items: Vec<PaperdollSlot> = Vec::new();
                for (index, item_id) in paperdoll.iter().enumerate() {
                    if *item_id == 0 {
                        continue;
                    }

                    let item = match ITEM_DB.items.get(*item_id as usize - 1) {
                        Some(item) => item,
                        None => {
                            continue;
                        }
                    };

                    if item.special == ItemSpecial::Cursed {
                        cursed_items.push(PaperdollSlot::from_index(index).unwrap());
                    }
                }

                if cursed_items.is_empty() {
                    return;
                }

                for slot in cursed_items.iter() {
                    character.destroy_equipment(slot);
                }

                character.calculate_stats();

                reply.item_type = ItemType::CureCurse;
                reply.item_type_data = Some(ItemReplyServerPacketItemTypeData::CureCurse(ItemReplyServerPacketItemTypeDataCureCurse {
                    stats: character.get_item_character_stats(),
                }));

                let visible_change = cursed_items.iter().any(|slot| slot.is_visible());
                if visible_change {
                    let packet = AvatarAgreeServerPacket {
                        change: AvatarChange {
                            player_id,
                            change_type: AvatarChangeType::Equipment,
                            sound: false,
                            change_type_data: Some(AvatarChangeChangeTypeData::Equipment(AvatarChangeChangeTypeDataEquipment {
                                equipment: character.get_paperdoll_bahws(),
                            })),
                        },
                    };

                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Agree,
                        PacketFamily::Avatar,
                        packet,
                    );
                }
            }
            _ => {
                return;
            }
        }

        let character = self.characters.get_mut(&player_id).unwrap();
        character.remove_item(item_id, 1);

        reply.used_item = Item {
            id: item_id,
            amount: match character.items.iter().find(|i| i.id == item_id) {
                Some(item) => item.amount,
                None => 0,
            },
        };

        reply.weight = character.get_weight();

        let mut writer = EoWriter::new();
        reply.serialize(&mut writer);

        player.send(PacketAction::Reply, PacketFamily::Item, writer.to_byte_array());
    }
}
