use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{
        server::character::{self, Reply},
        CharacterList, CharacterReply,
    },
};
use mysql_async::Conn;

use super::get_character_list;
use crate::{
    character::Character,
    errors::{WrongAccountError, WrongSessionIdError},
    player::PlayerHandle,
};

pub async fn delete_character(
    conn: &mut Conn,
    session_id: EOShort,
    character_id: EOInt,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    match player.take_session_id().await {
        Ok(actual_session_id) => {
            if actual_session_id != session_id {
                return Err(Box::new(WrongSessionIdError::new(
                    actual_session_id,
                    session_id,
                )));
            }

            let account_id = match player.get_account_id().await {
                Ok(account_id) => account_id,
                Err(e) => {
                    warn!(
                        "Tried to request character deletion with invalid state: {:?}",
                        e.actual
                    );
                    return Err(Box::new(e));
                }
            };

            let character = match Character::load(conn, character_id).await {
                Ok(character) => character,
                Err(e) => {
                    warn!(
                "Tried to request character deletion for a character that doesn't exist: {}",
                character_id
            );
                    return Err(e);
                }
            };

            if character.account_id != account_id {
                warn!(
                    "Player {} attempted to delete character ({}) belonging to another account: {}",
                    account_id, character.name, character.account_id
                );
                return Err(Box::new(WrongAccountError::new(
                    character.account_id,
                    account_id,
                )));
            }

            character.delete(conn).await?;
            let characters = get_character_list(conn, account_id).await?;

            Ok(Reply {
                reply_code: CharacterReply::Deleted,
                data: character::ReplyData::Deleted(character::ReplyDeleted {
                    character_list: CharacterList {
                        num_characters: characters.len() as EOChar,
                        characters,
                    },
                }),
            })
        }
        Err(e) => Err(Box::new(e)),
    }
}
