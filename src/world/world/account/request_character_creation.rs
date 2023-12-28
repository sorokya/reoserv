use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::server::{
    CharacterReply, CharacterReplyServerPacket, CharacterReplyServerPacketReplyCodeData,
    CharacterReplyServerPacketReplyCodeDataDefault, CharacterReplyServerPacketReplyCodeDataFull,
};
use eolib::protocol::net::{PacketAction, PacketFamily};

use super::get_num_of_characters::get_num_of_characters;

use super::super::World;

impl World {
    pub async fn request_character_creation(&self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let account_id = match player.get_account_id().await {
            Ok(account_id) => account_id,
            Err(e) => {
                player.close(format!("Error getting account_id: {}", e));
                return;
            }
        };

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                player.close(format!("Error getting connection from pool: {}", e));
                return;
            }
        };

        let num_of_characters = match get_num_of_characters(&mut conn, account_id).await {
            Ok(num_of_characters) => num_of_characters,
            Err(e) => {
                player.close(format!("Error getting number of characters: {}", e));
                return;
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
            reply.serialize(&mut writer);
            player.send(
                PacketAction::Reply,
                PacketFamily::Character,
                writer.to_byte_array(),
            );
            return;
        }

        let session_id = match player.generate_session_id().await {
            Ok(session_id) => session_id,
            Err(e) => {
                player.close(format!("Error generating session id: {}", e));
                return;
            }
        };

        let reply = CharacterReplyServerPacket {
            reply_code: CharacterReply::Unrecognized(session_id),
            reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Default(
                CharacterReplyServerPacketReplyCodeDataDefault::new(),
            )),
        };

        let mut writer = EoWriter::new();
        reply.serialize(&mut writer);
        player.send(
            PacketAction::Reply,
            PacketFamily::Character,
            writer.to_byte_array(),
        );
    }
}
