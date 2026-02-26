use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{PacketAction, PacketFamily, server::MessageCloseServerPacket},
};
use tokio::sync::oneshot;

use super::World;

impl World {
    pub async fn shutdown(&mut self, respond_to: oneshot::Sender<()>) {
        let packet = MessageCloseServerPacket::new();
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize MessageCloseServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        let sends = self.players.values().map(|player| {
            player.send_buf_await(PacketAction::Close, PacketFamily::Message, buf.clone())
        });
        let results = futures::future::join_all(sends).await;
        for result in results {
            if let Err(e) = result {
                error!("Failed to send shutdown packet to player: {}", e);
            }
        }

        self.save_async().await;

        let _ = respond_to.send(());
    }
}
