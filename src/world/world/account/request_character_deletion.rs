use eo::{data::EOInt, protocol::server::character::Player};
use tokio::sync::oneshot;

use crate::{character::Character, errors::WrongAccountError, player::PlayerHandle};

use super::super::World;

impl World {
    pub async fn request_character_deletion(
        &self,
        player: PlayerHandle,
        character_id: EOInt,
        respond_to: oneshot::Sender<Result<Player, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let account_id = match player.get_account_id().await {
            Ok(account_id) => account_id,
            Err(e) => {
                error!(
                    "Error getting account id: {}",
                    e
                );
                let _ = respond_to.send(Err(e));
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

        let session_id = player.generate_session_id().await;

        if let Err(e) = session_id {
            let _ = respond_to.send(Err(e));
            return;
        }

        let session_id = session_id.unwrap();

        let _ = respond_to.send(Ok(Player {
            session_id,
            character_id,
        }));
    }
}
