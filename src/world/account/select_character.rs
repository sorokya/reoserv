use eo::data::EOInt;
use mysql_async::Conn;

use crate::{character::Character, errors::WrongAccountError, player::PlayerHandle};

pub async fn select_character(
    conn: &mut Conn,
    character_id: EOInt,
    player: PlayerHandle,
) -> Result<Character, Box<dyn std::error::Error + Send + Sync>> {
    let account_id = match player.get_account_id().await {
        Ok(account_id) => account_id,
        Err(e) => {
            warn!(
                "Tried to select character with invalid state: {:?}",
                e.actual
            );
            return Err(Box::new(e));
        }
    };

    let mut character = match Character::load(conn, character_id).await {
        Ok(character) => character,
        Err(e) => {
            warn!(
                "Tried to select character that doesn't exist: {}",
                character_id
            );
            return Err(e);
        }
    };

    if character.account_id != account_id {
        warn!(
            "Player {} attempted to login to character ({}) belonging to another account: {}",
            account_id, character.name, character.account_id
        );
        return Err(Box::new(WrongAccountError::new(
            character.account_id,
            account_id,
        )));
    }

    let player_id = player.get_player_id().await;
    character.player_id = Some(player_id);
    character.player = Some(player);
    character.logged_in_at = Some(chrono::Utc::now());

    Ok(character)
}
