use eo::{
    data::Serializeable,
    protocol::{server::talk, PacketAction, PacketFamily, TalkReply},
};

use crate::player::PlayerHandle;

use super::super::World;

impl World {
    pub async fn send_private_message(&self, from: &PlayerHandle, to: &str, message: &str) {
        if let Ok(from_character) = from.get_character().await {
            match self.get_character_by_name(to).await {
                Ok(character) => send_private_message(
                    &from_character.name,
                    character.player.as_ref().unwrap(),
                    message,
                ),
                Err(_) => send_player_not_found(from, to),
            }
        }
    }
}

fn send_private_message(from: &str, to: &PlayerHandle, message: &str) {
    let packet = talk::Tell {
        message: message.to_string(),
        player_name: from.to_string(),
    };
    let buf = packet.serialize();
    to.send(PacketAction::Tell, PacketFamily::Talk, buf);
}

fn send_player_not_found(player: &PlayerHandle, to: &str) {
    let packet = talk::Reply {
        reply_code: TalkReply::NotFound,
        name: to.to_string(),
    };
    let buf = packet.serialize();
    player.send(PacketAction::Reply, PacketFamily::Talk, buf);
}
