use eo::{data::EOInt, net::packets::server::character::Player};
use mysql_async::Conn;

use super::WrongAccountError;
use crate::{character::Character, player::PlayerHandle};

pub async fn request_character_deletion(
    conn: &mut Conn,
    character_id: EOInt,
    player: PlayerHandle,
) -> Result<Player, Box<dyn std::error::Error + Send + Sync>> {
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

    Ok(Player {
        session_id: 1000,
        character_id,
    })
}
