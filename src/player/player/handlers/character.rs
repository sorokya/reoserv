use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        PacketAction, PacketFamily,
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
            CharacterReplyServerPacketReplyCodeDataFull,
            CharacterReplyServerPacketReplyCodeDataNotApproved,
            CharacterReplyServerPacketReplyCodeDataOk,
        },
    },
};

use crate::{
    SETTINGS,
    character::Character,
    db::{DbHandle, insert_params},
    errors::WrongSessionIdError,
    player::{
        ClientState,
        player::account::{get_character_list, get_num_of_characters},
    },
    utils::validate_character_name,
};

use super::super::Player;

impl Player {
    async fn character_create(&mut self, reader: EoReader) {
        let create = match CharacterCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                tracing::error!("Error deserializing CharacterCreateClientPacket {}", e);
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

        let session_id = match self.session_id {
            Some(session_id) => session_id,
            None => return,
        };

        if session_id != create.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(session_id, create.session_id)
            ))
            .await;
            return;
        }

        if !validate_character_name(&create.name.to_lowercase()) {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Character,
                    CharacterReplyServerPacket {
                        reply_code: CharacterReply::NotApproved,
                        reply_code_data: Some(
                            CharacterReplyServerPacketReplyCodeData::NotApproved(
                                CharacterReplyServerPacketReplyCodeDataNotApproved::new(),
                            ),
                        ),
                    },
                )
                .await;
            return;
        }

        let exists = match character_exists(&self.db, &create.name).await {
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
        if let Err(e) = character.save(&self.db).await {
            self.close(format!(
                "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                e, self.account_id, create
            ))
            .await;
            return;
        }

        tracing::info!("New character: {}", create.name);

        let characters = match get_character_list(&self.db, self.account_id).await {
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
                tracing::error!("Error deserializing CharacterRemoveClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
            return;
        }

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

        let character = match Character::load(&self.db, remove.character_id).await {
            Ok(character) => character,
            Err(e) => {
                self.close(format!(
                    "Failed to load character for deletion: {}\n\tcharacter_id: {}",
                    e, remove.character_id
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

        if let Err(e) = character.delete(&self.db).await {
            self.close(format!("Error deleting character: {}", e)).await;
            return;
        }

        let characters = match get_character_list(&self.db, self.account_id).await {
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
                tracing::error!("Error deserializing CharacterRemoveClientPacket {}", e);
                return;
            }
        };

        if request.request_string != "NEW" {
            return;
        }

        if self.state != ClientState::LoggedIn {
            return;
        }

        let num_of_characters = match get_num_of_characters(&self.db, self.account_id).await {
            Ok(num_of_characters) => num_of_characters,
            Err(e) => {
                self.close(format!("Error getting number of characters: {}", e))
                    .await;
                return;
            }
        };

        if num_of_characters >= SETTINGS.account.max_characters {
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
                tracing::error!("Error deserializing CharacterTakeClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
            return;
        }

        let character = match Character::load(&self.db, take.character_id).await {
            Ok(character) => character,
            Err(e) => {
                self.close(format!(
                    "Failed to load character for deletion: {}\n\tcharacter_id: {}",
                    e, take.character_id
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
            _ => tracing::error!("Unhandled packet Character_{:?}", action),
        }
    }
}

pub async fn character_exists(
    db: &DbHandle,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match db
        .query_one(&insert_params(
            r"SELECT id FROM `characters` WHERE `name` = :name",
            &[("name", &name.to_lowercase())],
        ))
        .await?
    {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}
