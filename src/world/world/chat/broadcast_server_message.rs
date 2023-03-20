use eo::{
    data::Serializeable,
    protocol::{server::talk, PacketAction, PacketFamily},
};

use crate::player::ClientState;

use super::super::World;

impl World {
    pub async fn broadcast_server_message(&self, message: &str) {
        let packet = talk::Server {
            message: message.to_string(),
        };
        let buf = packet.serialize();
        for player in self.players.values() {
            let state = player.get_state().await;
            if state == ClientState::Playing {
                player.send(PacketAction::Server, PacketFamily::Talk, buf.clone());
            }
        }
    }
}
