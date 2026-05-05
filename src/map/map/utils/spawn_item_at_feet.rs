use std::cmp;

use eolib::protocol::net::{PacketAction, PacketFamily, server::ItemAddServerPacket};

use crate::{ITEM_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn spawn_item_at_feet(&mut self, player_id: i32, item_id: i32, amount: i32) {
        let coords = match self.characters.get(&player_id) {
            Some(character) => character.coords,
            None => return,
        };

        if ITEM_DB.items.get(item_id as usize - 1).is_none() {
            return;
        }

        let amount = cmp::min(SETTINGS.load().limits.max_item, amount);

        let item_index = match self.add_item(item_id, amount, coords, 0, 0) {
            Ok(index) => index,
            Err(e) => {
                tracing::error!("Failed to add item to map: {}", e);
                return;
            }
        };

        self.send_packet_near(
            &coords,
            PacketAction::Add,
            PacketFamily::Item,
            ItemAddServerPacket {
                item_id,
                item_index,
                item_amount: amount,
                coords,
            },
        );
    }
}
