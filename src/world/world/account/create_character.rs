use crate::{character::Character, errors::WrongSessionIdError};
use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::client::CharacterCreateClientPacket;
use eolib::protocol::net::server::{
    CharacterReply, CharacterReplyServerPacket, CharacterReplyServerPacketReplyCodeData,
    CharacterReplyServerPacketReplyCodeDataExists, CharacterReplyServerPacketReplyCodeDataOk,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

use super::super::World;

use super::get_character_list::get_character_list;

impl World {
    pub async fn create_character(&self, player_id: i32, details: CharacterCreateClientPacket) {
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

            let session_id = player.take_session_id().await;
            if let Err(e) = session_id {
                player.close(format!("Error getting session id: {}", e));
                return;
            }

            let session_id = session_id.unwrap();

            if session_id != details.session_id {
                player.close(format!(
                    "{}",
                    WrongSessionIdError::new(session_id, details.session_id)
                ));
                return;
            }

            // TODO: validate name

            let exists = match character_exists(&mut conn, &details.name).await {
                Ok(exists) => exists,
                Err(e) => {
                    player.close(format!("Error checking if character exists: {}", e));
                    // Assume it exists if the check fails
                    true
                }
            };

            if exists {
                let reply = CharacterReplyServerPacket {
                    reply_code: CharacterReply::Exists,
                    reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Exists(
                        CharacterReplyServerPacketReplyCodeDataExists::new(),
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

            let account_id = match player.get_account_id().await {
                Ok(account_id) => account_id,
                Err(e) => {
                    player.close(format!("Error getting account_id: {}", e));
                    return;
                }
            };

            let mut character = Character::from_creation(account_id, &details);
            if let Err(e) = character.save(&mut conn).await {
                player.close(format!(
                    "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                    e, account_id, details
                ));
                return;
            }

            info!("New character: {}", details.name);

            let characters = get_character_list(&mut conn, account_id).await;
            if let Err(e) = characters {
                player.close(format!("Error getting character list: {}", e));
                return;
            }

            let characters = characters.unwrap();

            let reply = CharacterReplyServerPacket {
                reply_code: CharacterReply::OK,
                reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::OK(
                    CharacterReplyServerPacketReplyCodeDataOk { characters },
                )),
            };

            let mut writer = EoWriter::new();
            reply.serialize(&mut writer);
            player.send(
                PacketAction::Reply,
                PacketFamily::Character,
                writer.to_byte_array(),
            );
        });
    }
}

pub async fn character_exists(
    conn: &mut Conn,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match conn
        .exec_first::<Row, &str, Params>(
            r"SELECT id FROM `Character` WHERE `name` = :name",
            params! {
                "name" => name,
            },
        )
        .await?
    {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}
