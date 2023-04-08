use eo::{
    data::{EOChar, EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::{avatar, paperdoll},
        AvatarChange, AvatarChangeClothes, AvatarChangeData, AvatarSlot, PacketAction,
        PacketFamily,
    },
    pubs::EifItemType,
};

use crate::ITEM_DB;

use super::Map;

impl Map {
    pub fn unequip(&mut self, player_id: EOShort, item_id: EOShort, sub_loc: EOChar) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character");
                return;
            }
        };

        if !character.unequip(item_id, sub_loc) {
            return;
        }

        let change = AvatarChange {
            player_id,
            slot: AvatarSlot::Clothes,
            sound: 0,
            data: AvatarChangeData::Clothes(AvatarChangeClothes {
                paperdoll: character.get_paperdoll_bahws(),
            }),
        };

        let reply = paperdoll::Remove {
            change: change.clone(),
            item_id,
            sub_loc,
            stats: character.get_item_character_stats(),
        };

        debug!("{:?}", reply);

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        character.player.as_ref().unwrap().send(
            PacketAction::Remove,
            PacketFamily::Paperdoll,
            builder.get(),
        );

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

            debug!("{:?}", reply);

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            let buf = builder.get();

            for (target_player_id, character) in self.characters.iter() {
                if *target_player_id != player_id && character.is_in_range(&character.coords) {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Agree,
                        PacketFamily::Avatar,
                        buf.clone(),
                    );
                }
            }
        }
    }
}
