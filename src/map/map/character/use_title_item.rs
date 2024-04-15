use eolib::protocol::{
    net::{server::ItemReplyServerPacket, Item, PacketAction, PacketFamily},
    r#pub::ItemType,
};

use crate::{ITEM_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn use_title_item(&mut self, player_id: i32, item_id: i32, title: String) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                return;
            }
        };

        if !character.items.iter().any(|item| item.id == item_id) {
            return;
        }

        let item = match ITEM_DB.items.get(item_id as usize - 1) {
            Some(item) => item,
            None => {
                return;
            }
        };

        if item.r#type != ItemType::Reserved28 {
            return;
        }

        character.title = Some(title);

        if !SETTINGS.items.infinite_use_items.contains(&item_id) {
            character.remove_item(item_id, 1);
        }

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Reply,
                PacketFamily::Item,
                &ItemReplyServerPacket {
                    item_type: ItemType::Reserved28,
                    used_item: Item {
                        id: item_id,
                        amount: match character.items.iter().find(|i| i.id == item_id) {
                            Some(item) => item.amount,
                            None => 0,
                        },
                    },
                    weight: character.get_weight(),
                    item_type_data: None,
                },
            );
        }
    }
}
