use std::collections::HashMap;

use eo::{
    data::{EOShort, Serializeable},
    net::{packets::server::talk, Action, Family},
};

use crate::player::PlayerHandle;

pub async fn broadcast_announcement(
    name: &str,
    message: &str,
    players: &HashMap<EOShort, PlayerHandle>,
) {
    let packet = talk::Announce {
        name: name.to_string(),
        message: message.to_string(),
    };
    let buf = packet.serialize();
    for player in players.values() {
        if let Ok(character) = player.get_character().await {
            if character.name != name {
                player.send(Action::Announce, Family::Talk, buf.clone());
            }
        }
    }
}
