use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{server::message, PacketAction, PacketFamily},
};
use tokio::sync::oneshot;

use super::World;

impl World {
    pub async fn shutdown(&mut self, respond_to: oneshot::Sender<()>) {
        if let Some(maps) = self.maps.as_ref() {
            for map in maps.values() {
                map.save().await;
            }
        }

        let packet = message::Close::new();
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        let buf = builder.get();
        for player in self.players.values() {
            player.send(PacketAction::Close, PacketFamily::Message, buf.clone());
        }

        // wait a bit for the packets to be sent
        tokio::time::sleep(std::time::Duration::from_millis(100)).await;

        let _ = respond_to.send(());
    }
}
