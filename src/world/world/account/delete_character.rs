use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{
        server::character::{self, Reply},
        CharacterList, CharacterReply,
    },
};
use tokio::sync::oneshot;

use crate::{
    character::Character,
    errors::{WrongAccountError, WrongSessionIdError},
    player::PlayerHandle,
};

use super::super::World;

use super::get_character_list::get_character_list;

impl World {
    pub async fn delete_character(
        &self,
        player: PlayerHandle,
        session_id: EOShort,
        character_id: EOInt,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let actual_session_id = player.take_session_id().await;
        if let Err(e) = actual_session_id {
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let actual_session_id = actual_session_id.unwrap();
        if actual_session_id != session_id {
            let _ = respond_to.send(Err(Box::new(WrongSessionIdError::new(
                actual_session_id,
                session_id,
            ))));
            return;
        }

        let account_id = match player.get_account_id().await {
            Ok(account_id) => account_id,
            Err(e) => {
                warn!(
                    "Tried to request character deletion with invalid state: {:?}",
                    e.actual
                );
                let _ = respond_to.send(Err(Box::new(e)));
                return;
            }
        };

        let conn = self.pool.get_conn().await;

        if let Err(e) = conn {
            error!("Error getting connection from pool: {}", e);
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let mut conn = conn.unwrap();

        let character = match Character::load(&mut conn, character_id).await {
            Ok(character) => character,
            Err(e) => {
                warn!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    character_id
                );
                let _ = respond_to.send(Err(e));
                return;
            }
        };

        if character.account_id != account_id {
            warn!(
                "Player {} attempted to delete character ({}) belonging to another account: {}",
                account_id, character.name, character.account_id
            );
            let _ = respond_to.send(Err(Box::new(WrongAccountError::new(
                character.account_id,
                account_id,
            ))));
            return;
        }

        if let Err(e) = character.delete(&mut conn).await {
            error!("Error deleting character: {}", e);
            let _ = respond_to.send(Err(e));
            return;
        }

        let characters = get_character_list(&mut conn, account_id).await;

        if let Err(e) = characters {
            error!("Error getting character list: {}", e);
            let _ = respond_to.send(Err(e));
            return;
        }

        let characters = characters.unwrap();

        let _ = respond_to.send(Ok(Reply {
            reply_code: CharacterReply::Deleted,
            data: character::ReplyData::Deleted(character::ReplyDeleted {
                character_list: CharacterList {
                    num_characters: characters.len() as EOChar,
                    characters,
                },
            }),
        }));
    }
}
