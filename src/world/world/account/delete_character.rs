use eolib::data::{EoWriter, EoSerialize};
use eolib::protocol::net::{PacketAction, PacketFamily};
use eolib::protocol::net::server::{CharacterReplyServerPacket, CharacterReply, CharacterReplyServerPacketReplyCodeData, CharacterReplyServerPacketReplyCodeDataDeleted};

use crate::{character::Character, errors::WrongSessionIdError};

use super::super::World;

use super::get_character_list::get_character_list;

impl World {
    pub fn delete_character(&self, player_id: i32, session_id: i32, character_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.clone(),
            None => return,
        };

        let conn = self.pool.get_conn();

        tokio::spawn(async move {
            let mut conn = match conn.await {
                Ok(conn) => conn,
                Err(e) => {
                    player.close(format!("Error getting connection from pool: {}", e));
                    return;
                }
            };

            let actual_session_id = player.take_session_id().await;
            if let Err(e) = actual_session_id {
                player.close(format!("Error getting session id: {}", e));
                return;
            }

            let actual_session_id = actual_session_id.unwrap();
            if actual_session_id != session_id {
                error!(
                    "{}",
                    WrongSessionIdError::new(actual_session_id, session_id)
                );
                return;
            }

            let account_id = match player.get_account_id().await {
                Ok(account_id) => account_id,
                Err(e) => {
                    player.close(format!("Failed to get account id: {}", e));
                    return;
                }
            };

            let character = match Character::load(&mut conn, character_id).await {
                Ok(character) => character,
                Err(_) => {
                    player.close(format!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    character_id
                ));
                    return;
                }
            };

            if character.account_id != account_id {
                player.close(format!(
                    "Player {} attempted to delete character ({}) belonging to another account: {}",
                    account_id, character.name, character.account_id
                ));
                return;
            }

            if let Err(e) = character.delete(&mut conn).await {
                player.close(format!("Error deleting character: {}", e));
                return;
            }

            let characters = get_character_list(&mut conn, account_id).await;

            if let Err(e) = characters {
                player.close(format!("Error getting character list: {}", e));
                return;
            }

            let characters = characters.unwrap();

            let reply = CharacterReplyServerPacket {
                reply_code: CharacterReply::Deleted,
                reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Deleted(CharacterReplyServerPacketReplyCodeDataDeleted {
                    characters,
                })),
            };

            let mut writer = EoWriter::new();
            reply.serialize(&mut writer);
            player.send(PacketAction::Reply, PacketFamily::Character, writer.to_byte_array());
        });
    }
}
