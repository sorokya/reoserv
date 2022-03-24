use std::collections::HashMap;

use eo::{
    data::{EOShort, Serializeable},
    net::{packets::server::talk, Action, Family},
};

use crate::player::{PlayerHandle, State};

pub async fn broadcast_announcement(
    name: String,
    message: String,
    players: &HashMap<EOShort, PlayerHandle>,
) {
    let packet = talk::Announce {
        name: name.clone(),
        message: message.clone(),
    };
    let buf = packet.serialize();
    for player in players.values() {
        let state = player.get_state().await;
        if state == State::Playing {
            player.send(Action::Announce, Family::Talk, buf.clone());
        }
    }
}
