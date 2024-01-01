use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::server::{
    CharacterReply, CharacterReplyServerPacket, CharacterReplyServerPacketReplyCodeData,
    CharacterReplyServerPacketReplyCodeDataDeleted,
};
use eolib::protocol::net::{PacketAction, PacketFamily};

use crate::{character::Character, errors::WrongSessionIdError};

use super::super::Player;

use super::get_character_list::get_character_list;

impl Player {
    pub async fn delete_character(&mut self, session_id: i32, character_id: i32) -> bool {
        let conn = self.pool.get_conn();

        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return false;
            }
        };

        let actual_session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return false;
            }
        };

        if actual_session_id != session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(actual_session_id, session_id)
            ))
            .await;
            return false;
        }

        let character = match Character::load(&mut conn, character_id).await {
            Ok(character) => character,
            Err(_) => {
                self.close(format!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    character_id
                ))
                .await;
                return false;
            }
        };

        if character.account_id != self.account_id {
            self.close(format!(
                "Player {} attempted to delete character ({}) belonging to another account: {}",
                self.account_id, character.name, character.account_id
            ))
            .await;
            return false;
        }

        if let Err(e) = character.delete(&mut conn).await {
            self.close(format!("Error deleting character: {}", e)).await;
            return false;
        }

        let characters = match get_character_list(&mut conn, self.account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return false;
            }
        };

        let reply = CharacterReplyServerPacket {
            reply_code: CharacterReply::Deleted,
            reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Deleted(
                CharacterReplyServerPacketReplyCodeDataDeleted { characters },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            self.close(format!(
                "Failed to serialize CharacterReplyServerPacket: {}",
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
