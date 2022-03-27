use std::collections::HashMap;

use eo::{
    data::{EOShort, Serializeable},
    net::{packets::server::talk, Action, Family},
};

use crate::player::{PlayerHandle, State};

pub async fn broadcast_global_message(
    target_player_id: EOShort,
    name: &str,
    message: &str,
    players: &HashMap<EOShort, PlayerHandle>,
) {
    let packet = talk::Message {
        name: name.to_string(),
        message: message.to_string(),
    };
    let buf = packet.serialize();
    for player in players.values() {
        let state = player.get_state().await;
        let player_id = player.get_player_id().await;
        if state == State::Playing && player_id != target_player_id {
            player.send(Action::Message, Family::Talk, buf.clone());
        }
    }
}
