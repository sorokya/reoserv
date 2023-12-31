use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{TalkReply, TalkReplyServerPacket, TalkTellServerPacket},
        PacketAction, PacketFamily,
    },
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
    let packet = TalkTellServerPacket {
        message: message.to_string(),
        player_name: from.to_string(),
    };
    let mut writer = EoWriter::new();

    if let Err(e) = packet.serialize(&mut writer) {
        error!("Failed to serialize TalkTellServerPacket: {}", e);
        return;
    }

    to.send(
        PacketAction::Tell,
        PacketFamily::Talk,
        writer.to_byte_array(),
    );
}

fn send_player_not_found(player: &PlayerHandle, to: &str) {
    let packet = TalkReplyServerPacket {
        reply_code: TalkReply::NotFound,
        name: to.to_string(),
    };

    let mut writer = EoWriter::new();

    if let Err(e) = packet.serialize(&mut writer) {
        error!("Failed to serialize TalkReplyServerPacket: {}", e);
        return;
    }

    player.send(
        PacketAction::Reply,
        PacketFamily::Talk,
        writer.to_byte_array(),
    );
}
