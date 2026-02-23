use eolib::protocol::net::{server::ItemRemoveServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn remove_item(&mut self, item_index: i32) {
        if let Some(item) = self.items.remove(&item_index) {
            self.send_packet_near(
                &item.coords,
                PacketAction::Remove,
                PacketFamily::Item,
                ItemRemoveServerPacket { item_index },
            );
        }
    }
}
