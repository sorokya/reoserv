use std::cmp;

use eolib::protocol::net::{server::ItemAddServerPacket, PacketAction, PacketFamily};

use crate::{map::Item, ITEM_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub fn spawn_item_at_feet(&mut self, player_id: i32, item_id: i32, amount: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if ITEM_DB.items.get(item_id as usize - 1).is_none() {
            return;
        }

        let amount = cmp::min(SETTINGS.limits.max_item, amount);

        let item_index = self.get_next_item_index(1);

        self.items.insert(
            item_index,
            Item {
                id: item_id,
                amount,
                coords: character.coords,
                owner: 0,
                protected_ticks: 0,
            },
        );

        self.send_packet_near(
            &character.coords,
            PacketAction::Add,
            PacketFamily::Item,
            ItemAddServerPacket {
                item_id,
                item_index,
                item_amount: amount,
                coords: character.coords,
            },
        );
    }
}
