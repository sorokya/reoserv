use eolib::protocol::net::{PacketAction, PacketFamily, server::ItemRemoveServerPacket};

use super::super::Map;

impl Map {
    pub fn remove_item(&mut self, item_index: i32) {
        let item = match self.items.iter().find(|i| i.index == item_index) {
            Some(item) => item,
            None => return,
        };

        self.send_packet_near(
            &item.coords,
            PacketAction::Remove,
            PacketFamily::Item,
            ItemRemoveServerPacket { item_index },
        );

        self.items.retain(|i| i.index != item_index);
    }
}
