use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::MessageCloseServerPacket, PacketAction, PacketFamily},
};
use tokio::sync::oneshot;

use super::World;

impl World {
    pub async fn shutdown(&mut self, respond_to: oneshot::Sender<()>) {
        self.save().await;

        let packet = MessageCloseServerPacket::new();
        let mut writer = EoWriter::new();
        packet.serialize(&mut writer);
        let buf = writer.to_byte_array();
        for player in self.players.values() {
            player.send(PacketAction::Close, PacketFamily::Message, buf.clone());
        }

        // wait a bit for the packets to be sent
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let _ = respond_to.send(());
    }
}
