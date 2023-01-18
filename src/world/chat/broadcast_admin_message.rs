use std::collections::HashMap;

use eo::{
    data::{EOChar, EOShort, Serializeable},
    protocol::{server::talk, AdminLevel, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

pub async fn broadcast_admin_message(
    name: &str,
    message: &str,
    players: &HashMap<EOShort, PlayerHandle>,
) {
    let packet = talk::Admin {
        player_name: name.to_string(),
        message: message.to_string(),
    };
    let buf = packet.serialize();
    for player in players.values() {
        if let Ok(character) = player.get_character().await {
            if character.name != name
                && character.admin_level as EOChar >= AdminLevel::Guardian as EOChar
            {
                player.send(PacketAction::Admin, PacketFamily::Talk, buf.clone());
            }
        }
    }
}
