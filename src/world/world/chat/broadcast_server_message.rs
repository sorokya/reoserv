use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{server::talk, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn broadcast_server_message(&self, message: &str) {
        let packet = talk::Server {
            message: message.to_string(),
        };
        let mut builder = StreamBuilder::new();
        packet.serialize(&mut builder);
        let buf = builder.get();
        for player in self.players.values() {
            player.send(PacketAction::Server, PacketFamily::Talk, buf.clone());
        }
    }
}
