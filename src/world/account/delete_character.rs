use eo::{
    data::{EOChar, EOInt, EOShort},
    net::{packets::server::character::Reply, CharacterList},
};
use mysql_async::Conn;

use super::{get_character_list, WrongAccountError};
use crate::{character::Character, player::PlayerHandle};

pub async fn delete_character(
    conn: &mut Conn,
    _session_id: EOShort,
    character_id: EOInt,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: validate session id

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
    Ok(Reply::deleted(CharacterList {
        length: characters.len() as EOChar,
        unknown: 1,
        characters,
    }))
}
