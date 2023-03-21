use eo::data::EOInt;
use eo::protocol::server::welcome::{Reply, ReplyData};
use eo::protocol::WelcomeReply;
use tokio::sync::oneshot;

use crate::{character::Character, errors::WrongAccountError, player::PlayerHandle};

use super::super::World;

use super::calculate_stats::calculate_stats;

impl World {
    pub async fn select_character(
        &mut self,
        player: PlayerHandle,
        character_id: EOInt,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
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

        let mut character = match Character::load(&mut conn, character_id).await {
            Ok(character) => character,
            Err(e) => {
                warn!(
                    "Tried to select character that doesn't exist: {}",
                    character_id
                );
                let _ = respond_to.send(Err(e));
                return;
            }
        };

        if character.account_id != account_id {
            warn!(
                "Player {} attempted to login to character ({}) belonging to another account: {}",
                account_id, character.name, character.account_id
            );
            let _ = respond_to.send(Err(Box::new(WrongAccountError::new(
                character.account_id,
                account_id,
            ))));
            return;
        }

        let player_id = player.get_player_id().await;

        if let Err(e) = player_id {
            let _ = respond_to.send(Err(e));
            return;
        }

        let player_id = player_id.unwrap();

        character.player_id = Some(player_id);
        character.player = Some(player.clone());
        character.logged_in_at = Some(chrono::Utc::now());

        // TODO: move this to Character::calculate_stats
        calculate_stats(&mut character);

        let select_character = match self
            .get_welcome_request_data(player.clone(), &character)
            .await
        {
            Ok(select_character) => select_character,
            Err(err) => {
                let _ = respond_to.send(Err(err));
                return;
            }
        };

        self.characters
            .insert(character.name.to_string(), player_id);
        player.set_character(Box::new(character));

        let _ = respond_to.send(Ok(Reply {
            reply_code: WelcomeReply::SelectCharacter,
            data: ReplyData::SelectCharacter(select_character),
        }));
    }
}
