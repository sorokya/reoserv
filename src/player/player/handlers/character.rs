use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            CharacterCreateClientPacket, CharacterRemoveClientPacket, CharacterRequestClientPacket,
            CharacterTakeClientPacket,
        },
        server::{
            CharacterPlayerServerPacket, CharacterReply, CharacterReplyServerPacket,
            CharacterReplyServerPacketReplyCodeData,
            CharacterReplyServerPacketReplyCodeDataDefault,
            CharacterReplyServerPacketReplyCodeDataDeleted,
            CharacterReplyServerPacketReplyCodeDataExists,
            CharacterReplyServerPacketReplyCodeDataFull, CharacterReplyServerPacketReplyCodeDataOk,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

use crate::{
    character::Character,
    errors::WrongSessionIdError,
    player::{
        player::account::{get_character_list, get_num_of_characters},
        ClientState,
    },
    SETTINGS,
};

use super::super::Player;

impl Player {
    async fn character_create(&mut self, reader: EoReader) {
        let create = match CharacterCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                error!("Error deserializing CharacterCreateClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
            return;
        }

        if create.hair_color < 0
            || create.hair_color > SETTINGS.character.max_hair_color
            || create.hair_style < 0
            || create.hair_style > SETTINGS.character.max_hair_style
        {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return;
            }
        };

        if session_id != create.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(session_id, create.session_id)
            ))
            .await;
            return;
        }

        // TODO: validate name

        let exists = match character_exists(&mut conn, &create.name).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if character exists: {}", e))
                    .await;
                // Assume it exists if the check fails
                true
            }
        };

        if exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Character,
                    CharacterReplyServerPacket {
                        reply_code: CharacterReply::Exists,
                        reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Exists(
                            CharacterReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;

            return;
        }

        let mut character = Character::from_creation(self.account_id, &create);
        if let Err(e) = character.save(&mut conn).await {
            self.close(format!(
                "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                e, self.account_id, create
            ))
            .await;
            return;
        }

        info!("New character: {}", create.name);

        let characters = match get_character_list(&mut conn, self.account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return;
            }
        };

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Character,
                CharacterReplyServerPacket {
                    reply_code: CharacterReply::OK,
                    reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::OK(
                        CharacterReplyServerPacketReplyCodeDataOk { characters },
                    )),
                },
            )
            .await;
    }

    async fn character_remove(&mut self, reader: EoReader) {
        let remove = match CharacterRemoveClientPacket::deserialize(&reader) {
            Ok(remove) => remove,
            Err(e) => {
                error!("Error deserializing CharacterRemoveClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
            return;
        }

        let conn = self.pool.get_conn();

        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let actual_session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return;
            }
        };

        if actual_session_id != remove.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(actual_session_id, remove.session_id)
            ))
            .await;
            return;
        }

        let character = match Character::load(&mut conn, remove.character_id).await {
            Ok(character) => character,
            Err(_) => {
                self.close(format!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    remove.character_id
                ))
                .await;
                return;
            }
        };

        if character.account_id != self.account_id {
            self.close(format!(
                "Player {} attempted to delete character ({}) belonging to another account: {}",
                self.account_id, character.name, character.account_id
            ))
            .await;
            return;
        }

        if let Err(e) = character.delete(&mut conn).await {
            self.close(format!("Error deleting character: {}", e)).await;
            return;
        }

        let characters = match get_character_list(&mut conn, self.account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return;
            }
        };

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Character,
                CharacterReplyServerPacket {
                    reply_code: CharacterReply::Deleted,
                    reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Deleted(
                        CharacterReplyServerPacketReplyCodeDataDeleted { characters },
                    )),
                },
            )
            .await;
    }

    async fn character_request(&mut self, reader: EoReader) {
        let request = match CharacterRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing CharacterRemoveClientPacket {}", e);
                return;
            }
        };

        if request.request_string != "NEW" {
            return;
        }

        if self.state != ClientState::LoggedIn {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let num_of_characters = match get_num_of_characters(&mut conn, self.account_id).await {
            Ok(num_of_characters) => num_of_characters,
            Err(e) => {
                self.close(format!("Error getting number of characters: {}", e))
                    .await;
                return;
            }
        };

        // TODO: configurable max number of characters?
        if num_of_characters >= 3 {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Character,
                    CharacterReplyServerPacket {
                        reply_code: CharacterReply::Full,
                        reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Full(
                            CharacterReplyServerPacketReplyCodeDataFull::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let session_id = self.generate_session_id();

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Character,
                CharacterReplyServerPacket {
                    reply_code: CharacterReply::Unrecognized(session_id),
                    reply_code_data: Some(CharacterReplyServerPacketReplyCodeData::Default(
                        CharacterReplyServerPacketReplyCodeDataDefault::new(),
                    )),
                },
            )
            .await;
    }

    async fn character_take(&mut self, reader: EoReader) {
        let take = match CharacterTakeClientPacket::deserialize(&reader) {
            Ok(take) => take,
            Err(e) => {
                error!("Error deserializing CharacterTakeClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let character = match Character::load(&mut conn, take.character_id).await {
            Ok(character) => character,
            Err(_) => {
                self.close(format!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    take.character_id
                ))
                .await;
                return;
            }
        };

        if character.account_id != self.account_id {
            self.close(format!(
                "Player {} attempted to delete character ({}) belonging to another account: {}",
                self.account_id, character.name, character.account_id
            ))
            .await;
            return;
        }

        let session_id = self.generate_session_id();

        let _ = self
            .bus
            .send(
                PacketAction::Player,
                PacketFamily::Character,
                CharacterPlayerServerPacket {
                    session_id,
                    character_id: take.character_id,
                },
            )
            .await;
    }

    pub async fn handle_character(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Create => self.character_create(reader).await,
            PacketAction::Remove => self.character_remove(reader).await,
            PacketAction::Request => self.character_request(reader).await,
            PacketAction::Take => self.character_take(reader).await,
            _ => error!("Unhandled packet Character_{:?}", action),
        }
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
