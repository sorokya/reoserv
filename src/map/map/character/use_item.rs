use eo::{
    data::{i32, EOInt, i32, Serializeable, StreamBuilder},
    protocol::{
        server::{
            avatar,
            item::{self, ReplyCureCurse, ReplyData, ReplyEffectPotion, ReplyHairDye, ReplyHeal},
            recover,
        },
        AvatarChange, AvatarChangeClothes, AvatarChangeData, AvatarChangeHairColor, AvatarSlot,
        Coords, Item, ItemType, PacketAction, PacketFamily, WarpAnimation,
    },
    pubs::{EifItemSpecial, EifItemType},
};

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

        let mut reply = item::Reply::default();
        let player = character.player.as_ref().unwrap().clone();

        match item.r#type {
            EifItemType::Heal => {
                let hp_gain = character.heal(item.hp);
                let tp_gain = character.tp_heal(item.tp);
                if hp_gain == 0 && tp_gain == 0 {
                    return;
                }
                reply.data = ReplyData::Heal(ReplyHeal {
                    hp_gain: hp_gain as EOInt,
                    hp: character.hp,
                    tp: character.tp,
                });
                reply.used_item_type = ItemType::Heal;

                if hp_gain > 0 {
                    let packet = recover::Agree {
                        player_id,
                        heal_hp: hp_gain as EOInt,
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
            EifItemType::Teleport => {
                if self.file.can_scroll == 0 {
                    return;
                }

                player.request_warp(
                    item.spec1 as i32,
                    Coords {
                        x: item.spec2 as i32,
                        y: item.spec3 as i32,
                    },
                    character.map_id == item.spec1 as i32,
                    Some(WarpAnimation::Scroll),
                );
                reply.used_item_type = ItemType::Teleport;
            }
            EifItemType::Spell => todo!(),
            EifItemType::EXPReward => todo!(),
            EifItemType::StatReward => todo!(),
            EifItemType::SkillReward => todo!(),
            EifItemType::Beer => {
                reply.used_item_type = ItemType::Beer;
            }
            EifItemType::EffectPotion => {
                reply.used_item_type = ItemType::EffectPotion;
                reply.data = ReplyData::EffectPotion(ReplyEffectPotion {
                    effect_id: item.spec1 as i32,
                });
                self.play_effect(player_id, item.spec1);
            }
            EifItemType::HairDye => {
                reply.used_item_type = ItemType::HairDye;
                reply.data = ReplyData::HairDye(ReplyHairDye {
                    hair_color: item.spec1 as i32,
                });
                character.hair_color = item.spec1 as i32;
                let packet = avatar::Agree {
                    change: AvatarChange {
                        player_id,
                        slot: AvatarSlot::HairColor,
                        sound: 0,
                        data: AvatarChangeData::HairColor(AvatarChangeHairColor {
                            color: item.spec1 as i32,
                        }),
                    },
                };
                self.send_packet_near_player(
                    player_id,
                    PacketAction::Agree,
                    PacketFamily::Avatar,
                    packet,
                );
            }
            EifItemType::CureCurse => {
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

                    if item.special == EifItemSpecial::Cursed {
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

                reply.used_item_type = ItemType::CureCurse;
                reply.data = ReplyData::CureCurse(ReplyCureCurse {
                    stats: character.get_item_character_stats(),
                });

                let visible_change = cursed_items.iter().any(|slot| slot.is_visible());
                if visible_change {
                    let packet = avatar::Agree {
                        change: AvatarChange {
                            player_id,
                            slot: AvatarSlot::Clothes,
                            sound: 0,
                            data: AvatarChangeData::Clothes(AvatarChangeClothes {
                                paperdoll: character.get_paperdoll_bahws(),
                            }),
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

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(PacketAction::Reply, PacketFamily::Item, builder.get());
    }
}
