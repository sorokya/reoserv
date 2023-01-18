use std::collections::HashMap;

use eo::{
    data::{EOShort, Serializeable},
    protocol::{server::talk, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

pub async fn broadcast_server_message(message: &str, players: &HashMap<EOShort, PlayerHandle>) {
    let packet = talk::Server {
        message: message.to_string(),
    };
    let buf = packet.serialize();
    for player in players.values() {
        let state = player.get_state().await;
        if state == State::Playing {
            player.send(PacketAction::Server, PacketFamily::Talk, buf.clone());
        }
    }
}
