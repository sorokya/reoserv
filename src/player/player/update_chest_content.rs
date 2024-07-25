use bytes::Bytes;
use eolib::protocol::net::{PacketAction, PacketFamily};

use super::Player;

impl Player {
    pub async fn update_chest_content(&mut self, chest_index: usize, buf: Bytes) {
        if self.chest_index == Some(chest_index) {
            let _ = self
                .bus
                .send_buf(PacketAction::Agree, PacketFamily::Chest, buf)
                .await;
        }
    }
}
