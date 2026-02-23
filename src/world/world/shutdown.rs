use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::MessageCloseServerPacket, PacketAction, PacketFamily},
};
use tokio::sync::oneshot;

use super::World;

impl World {
    pub async fn shutdown(&mut self, respond_to: oneshot::Sender<()>) {
        self.save();

        let packet = MessageCloseServerPacket::new();

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize MessageCloseServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for player in self.players.values() {
            player.send_buf(PacketAction::Close, PacketFamily::Message, buf.clone());
        }

        // wait a bit for the packets to be sent and maps to save
        tokio::time::sleep(std::time::Duration::from_secs(1)).await;

        let _ = respond_to.send(());
    }
}
