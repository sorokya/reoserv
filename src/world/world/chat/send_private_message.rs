use eolib::protocol::net::{
    server::{TalkReply, TalkReplyServerPacket, TalkTellServerPacket},
    PacketAction, PacketFamily,
};

use crate::player::PlayerHandle;

use super::super::World;

impl World {
    pub async fn send_private_message(&self, from: &PlayerHandle, to: &str, message: &str) {
        if let Ok(fromacter) = from.get_character().await {
            match self.get_character_by_name(to).await {
                Ok(character) => send_private_message(
                    &fromacter.name,
                    character.player.as_ref().unwrap(),
                    message,
                ),
                Err(_) => send_player_not_found(from, to),
            }
        }
    }
}

fn send_private_message(from: &str, to: &PlayerHandle, message: &str) {
    to.send(
        PacketAction::Tell,
        PacketFamily::Talk,
        &TalkTellServerPacket {
            message: message.to_string(),
            player_name: from.to_string(),
        },
    );
}

fn send_player_not_found(player: &PlayerHandle, to: &str) {
    player.send(
        PacketAction::Reply,
        PacketFamily::Talk,
        &TalkReplyServerPacket {
            reply_code: TalkReply::NotFound,
            name: to.to_string(),
        },
    );
}
