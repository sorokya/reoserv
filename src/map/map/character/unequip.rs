use eo::{
    data::{i32, EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::{avatar, paperdoll},
        AvatarChange, AvatarChangeClothes, AvatarChangeData, AvatarSlot, PacketAction,
        PacketFamily,
    },
    pubs::EifItemType,
};

use crate::ITEM_DB;

use super::super::Map;

impl Map {
    pub fn unequip(&mut self, player_id: EOShort, item_id: EOShort, sub_loc: i32) {
        let target = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character");
                return;
            }
        };

        if !target.unequip(item_id, sub_loc) {
            return;
        }

        let change = AvatarChange {
            player_id,
            slot: AvatarSlot::Clothes,
            sound: 0,
            data: AvatarChangeData::Clothes(AvatarChangeClothes {
                paperdoll: target.get_paperdoll_bahws(),
            }),
        };

        let reply = paperdoll::Remove {
            change: change.clone(),
            item_id,
            sub_loc,
            stats: target.get_item_character_stats(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        target.player.as_ref().unwrap().send(
            PacketAction::Remove,
            PacketFamily::Paperdoll,
            builder.get(),
        );

        if target.hidden {
            return;
        }

        let is_visible_change = matches!(
            ITEM_DB.items.get(item_id as usize - 1).unwrap().r#type,
            EifItemType::Armor
                | EifItemType::Weapon
                | EifItemType::Shield
                | EifItemType::Hat
                | EifItemType::Boots
        );

        if is_visible_change && self.characters.len() > 1 {
            let reply = avatar::Agree { change };

            self.send_packet_near_player(
                player_id,
                PacketAction::Agree,
                PacketFamily::Avatar,
                reply,
            );
        }
    }
}
