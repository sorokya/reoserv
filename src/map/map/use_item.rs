use eo::{data::{EOShort, StreamBuilder, Serializeable, EOChar, EOInt}, pubs::EifItemType, protocol::{server::item::{self, ReplyData, ReplyHeal, ReplyEffectPotion, ReplyHairDye}, PacketAction, PacketFamily, Coords, WarpAnimation, Item, Weight, ItemType}};

use crate::ITEM_DB;

use super::Map;

impl Map {
    pub fn use_item(&mut self, player_id: EOShort, item_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                return;
            },
        };

        if !character.items.iter().any(|item| item.id == item_id) {
            return;
        }

        let item = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => {
                return;
            },
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
            },
            EifItemType::Teleport => {
                // TODO: verify that the map exists
                if self.file.can_scroll == 0 {
                    return;
                }

                player.request_warp(item.spec1 as EOShort, Coords {
                    x: item.spec2 as EOChar,
                    y: item.spec3 as EOChar,
                }, character.map_id == item.spec1 as EOShort, Some(WarpAnimation::Scroll));
                reply.used_item_type = ItemType::Teleport;
            },
            EifItemType::Spell => todo!(),
            EifItemType::EXPReward => todo!(),
            EifItemType::StatReward => todo!(),
            EifItemType::SkillReward => todo!(),
            EifItemType::Beer => {
                reply.used_item_type = ItemType::Beer;
            },
            EifItemType::EffectPotion => {
                reply.used_item_type = ItemType::EffectPotion;
                reply.data = ReplyData::EffectPotion(ReplyEffectPotion {
                    effect_id: item.spec1 as EOShort,
                });
                self.play_effect(player_id, item.spec1);
            },
            EifItemType::HairDye => {
                reply.used_item_type = ItemType::HairDye;
                reply.data = ReplyData::HairDye(ReplyHairDye {
                    hair_color: item.spec1 as EOChar,
                });
                character.hair_color = item.spec1 as EOShort;
                // TODO: send packet to update hair color for other players
            },
            EifItemType::CureCurse => todo!(),
            _ => {
                return;
            },
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

        reply.weight = Weight {
            current: character.weight,
            max: character.max_weight,
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(PacketAction::Reply, PacketFamily::Item, builder.get());
    }
}