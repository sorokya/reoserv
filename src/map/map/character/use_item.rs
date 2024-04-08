use eolib::protocol::{
    net::{
        server::{
            AvatarAgreeServerPacket, AvatarChange, AvatarChangeChangeTypeData,
            AvatarChangeChangeTypeDataEquipment, AvatarChangeChangeTypeDataHairColor,
            AvatarChangeType, ItemReplyServerPacket, ItemReplyServerPacketItemTypeData,
            ItemReplyServerPacketItemTypeDataCureCurse,
            ItemReplyServerPacketItemTypeDataEffectPotion,
            ItemReplyServerPacketItemTypeDataHairDye, ItemReplyServerPacketItemTypeDataHeal,
            RecoverAgreeServerPacket, WarpEffect,
        },
        Item, PacketAction, PacketFamily,
    },
    r#pub::{ItemSpecial, ItemType},
    Coords,
};

use crate::{character::EquipmentSlot, INN_DB, ITEM_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn use_item(&mut self, player_id: i32, item_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                return;
            }
        };

        // TODO: Validate in player thread
        {
            let player = match character.player.as_ref() {
                Some(player) => player,
                None => return,
            };
            if player.is_trading().await {
                return;
            }
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

        let mut packet = ItemReplyServerPacket::default();

        match item.r#type {
            ItemType::Heal => {
                let hp_gain = character.heal(item.hp);
                let tp_gain = character.tp_heal(item.tp);
                if hp_gain == 0 && tp_gain == 0 {
                    return;
                }
                packet.item_type_data = Some(ItemReplyServerPacketItemTypeData::Heal(
                    ItemReplyServerPacketItemTypeDataHeal {
                        hp_gain,
                        hp: character.hp,
                        tp: character.tp,
                    },
                ));
                packet.item_type = ItemType::Heal;

                if hp_gain > 0 {
                    let packet = RecoverAgreeServerPacket {
                        player_id,
                        heal_hp: hp_gain,
                        hp_percentage: character.get_hp_percentage(),
                    };

                    if let Some(player) = character.player.as_ref() {
                        player.update_party_hp(character.get_hp_percentage());
                    }

                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Agree,
                        PacketFamily::Recover,
                        &packet,
                    );
                }
            }
            ItemType::Teleport => {
                if !self.file.can_scroll {
                    return;
                }

                let (map_id, coords) = {
                    match item.spec1 {
                        0 => match INN_DB.inns.iter().find(|inn| inn.name == character.home) {
                            Some(inn) => (
                                inn.spawn_map,
                                Coords {
                                    x: inn.spawn_x,
                                    y: inn.spawn_y,
                                },
                            ),
                            None => (
                                SETTINGS.rescue.map,
                                Coords {
                                    x: SETTINGS.rescue.x,
                                    y: SETTINGS.rescue.y,
                                },
                            ),
                        },
                        _ => (
                            item.spec1,
                            Coords {
                                x: item.spec2,
                                y: item.spec3,
                            },
                        ),
                    }
                };

                if let Some(player) = character.player.as_ref() {
                    player.request_warp(
                        map_id,
                        coords,
                        character.map_id == map_id,
                        Some(WarpEffect::Scroll),
                    );
                }
                packet.item_type = ItemType::Teleport;
            }
            ItemType::Alcohol => {
                packet.item_type = ItemType::Alcohol;
            }
            ItemType::EffectPotion => {
                packet.item_type = ItemType::EffectPotion;
                packet.item_type_data = Some(ItemReplyServerPacketItemTypeData::EffectPotion(
                    ItemReplyServerPacketItemTypeDataEffectPotion {
                        effect_id: item.spec1,
                    },
                ));
                self.effect_on_players(&[player_id], item.spec1);
            }
            ItemType::HairDye => {
                packet.item_type = ItemType::HairDye;
                packet.item_type_data = Some(ItemReplyServerPacketItemTypeData::HairDye(
                    ItemReplyServerPacketItemTypeDataHairDye {
                        hair_color: item.spec1,
                    },
                ));
                character.hair_color = item.spec1;
                let packet = AvatarAgreeServerPacket {
                    change: AvatarChange {
                        player_id,
                        change_type: AvatarChangeType::HairColor,
                        sound: false,
                        change_type_data: Some(AvatarChangeChangeTypeData::HairColor(
                            AvatarChangeChangeTypeDataHairColor {
                                hair_color: item.spec1,
                            },
                        )),
                    },
                };
                self.send_packet_near_player(
                    player_id,
                    PacketAction::Agree,
                    PacketFamily::Avatar,
                    &packet,
                );
            }
            ItemType::CureCurse => {
                let paperdoll = character.get_equipment_array();
                let mut cursed_items: Vec<EquipmentSlot> = Vec::new();
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
                        cursed_items.push(EquipmentSlot::from_index(index).unwrap());
                    }
                }

                if cursed_items.is_empty() {
                    return;
                }

                for slot in cursed_items.iter() {
                    character.destroy_equipment(slot);
                }

                character.calculate_stats();

                packet.item_type = ItemType::CureCurse;
                packet.item_type_data = Some(ItemReplyServerPacketItemTypeData::CureCurse(
                    ItemReplyServerPacketItemTypeDataCureCurse {
                        stats: character.get_character_stats_equipment_change(),
                    },
                ));

                let visible_change = cursed_items.iter().any(|slot| slot.is_visible());
                if visible_change {
                    let packet = AvatarAgreeServerPacket {
                        change: AvatarChange {
                            player_id,
                            change_type: AvatarChangeType::Equipment,
                            sound: false,
                            change_type_data: Some(AvatarChangeChangeTypeData::Equipment(
                                AvatarChangeChangeTypeDataEquipment {
                                    equipment: character.get_equipment_change(),
                                },
                            )),
                        },
                    };

                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Agree,
                        PacketFamily::Avatar,
                        &packet,
                    );
                }
            }
            _ => {
                return;
            }
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !SETTINGS.items.infinite_use_items.contains(&item_id) {
            character.remove_item(item_id, 1);
        }

        if let Some(player) = character.player.as_ref() {
            packet.used_item = Item {
                id: item_id,
                amount: match character.items.iter().find(|i| i.id == item_id) {
                    Some(item) => item.amount,
                    None => 0,
                },
            };

            packet.weight = character.get_weight();

            player.send(PacketAction::Reply, PacketFamily::Item, &packet);
        }
    }
}
