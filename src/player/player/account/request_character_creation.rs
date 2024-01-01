use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::server::{
    CharacterReply, CharacterReplyServerPacket, CharacterReplyServerPacketReplyCodeData,
    CharacterReplyServerPacketReplyCodeDataDefault, CharacterReplyServerPacketReplyCodeDataFull,
};
use eolib::protocol::net::{PacketAction, PacketFamily};

use super::get_num_of_characters::get_num_of_characters;

use super::super::Player;

impl Player {
    pub async fn request_character_creation(&mut self) -> bool {
        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return false;
            }
        };

        let num_of_characters = match get_num_of_characters(&mut conn, self.account_id).await {
            Ok(num_of_characters) => num_of_characters,
            Err(e) => {
                self.close(format!("Error getting number of characters: {}", e))
                    .await;
                return false;
            }
        };

        // TODO: configurable max number of characters?
        if num_of_characters >= 3 {
            let reply = CharacterReplyServerPacket {
                reply_code: CharacterReply::Full,
                reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Full(
                    CharacterReplyServerPacketReplyCodeDataFull::new(),
                )),
            };
            let mut writer = EoWriter::new();

            if let Err(e) = reply.serialize(&mut writer) {
                self.close(format!(
                    "Error serializing CharacterReplyServerPacket: {}",
                    e
                ))
                .await;
                return false;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Character,
                    writer.to_byte_array(),
                )
                .await;
            return true;
        }

        let session_id = self.generate_session_id();

        let reply = CharacterReplyServerPacket {
            reply_code: CharacterReply::Unrecognized(session_id),
            reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Default(
                CharacterReplyServerPacketReplyCodeDataDefault::new(),
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            self.close(format!(
                "Error serializing CharacterReplyServerPacket: {}",
                e
            ))
            .await;
            return false;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Character,
                writer.to_byte_array(),
            )
            .await;

        true
    }
}
