use crate::SETTINGS;
use crate::{character::Character, errors::WrongSessionIdError};
use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::client::CharacterCreateClientPacket;
use eolib::protocol::net::server::{
    CharacterReply, CharacterReplyServerPacket, CharacterReplyServerPacketReplyCodeData,
    CharacterReplyServerPacketReplyCodeDataExists, CharacterReplyServerPacketReplyCodeDataOk,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

use super::super::Player;

use super::get_character_list::get_character_list;

impl Player {
    pub async fn create_character(&mut self, packet: CharacterCreateClientPacket) -> bool {
        if packet.hair_color < 0
            || packet.hair_color > SETTINGS.character.max_hair_color
            || packet.hair_style < 0
            || packet.hair_style > SETTINGS.character.max_hair_style
        {
            return true;
        }

        let conn = self.pool.get_conn();

        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return false;
            }
        };

        let session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return false;
            }
        };

        if session_id != packet.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(session_id, packet.session_id)
            ))
            .await;
            return false;
        }

        // TODO: validate name

        let exists = match character_exists(&mut conn, &packet.name).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if character exists: {}", e))
                    .await;
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

            return true;
        }

        let mut character = Character::from_creation(self.account_id, &packet);
        if let Err(e) = character.save(&mut conn).await {
            self.close(format!(
                "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                e, self.account_id, packet
            ))
            .await;
            return false;
        }

        info!("New character: {}", packet.name);

        let characters = match get_character_list(&mut conn, self.account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return false;
            }
        };

        let reply = CharacterReplyServerPacket {
            reply_code: CharacterReply::OK,
            reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::OK(
                CharacterReplyServerPacketReplyCodeDataOk { characters },
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
