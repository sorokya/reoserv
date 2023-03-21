use eo::protocol::{
    server::character::{self, Reply},
    CharacterReply,
};
use tokio::sync::oneshot;

use crate::player::PlayerHandle;

use super::get_num_of_characters::get_num_of_characters;

use super::super::World;

impl World {
    pub async fn request_character_creation(
        &self,
        player: PlayerHandle,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let account_id = match player.get_account_id().await {
            Ok(account_id) => account_id,
            Err(e) => {
                error!("Error getting account_id: {}", e);
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

        let num_of_characters = get_num_of_characters(&mut conn, account_id).await;

        if let Err(e) = num_of_characters {
            error!("Error getting number of characters: {}", e);
            let _ = respond_to.send(Err(e));
            return;
        }

        let num_of_characters = num_of_characters.unwrap();

        // TODO: configurable max number of characters?
        if num_of_characters >= 3 {
            let _ = respond_to.send(Ok(Reply {
                reply_code: CharacterReply::Full,
                data: character::ReplyData::Full(character::ReplyFull {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let session_id = player.generate_session_id().await;

        if let Err(e) = session_id {
            let _ = respond_to.send(Err(e));
            return;
        }

        let session_id = session_id.unwrap();

        let _ = respond_to.send(Ok(Reply {
            reply_code: CharacterReply::SessionId(session_id),
            data: character::ReplyData::SessionId(character::ReplySessionId {
                ok: "OK".to_string(),
            }),
        }));
    }
}
