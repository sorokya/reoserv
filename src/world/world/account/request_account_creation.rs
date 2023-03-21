use eo::{
    data::EOChar,
    protocol::{
        server::account::{self, Reply},
        AccountReply,
    },
};
use tokio::sync::oneshot;

use crate::player::PlayerHandle;

use super::account_exists::account_exists;

use super::super::World;

impl World {
    pub async fn request_account_creation(
        &self,
        player: PlayerHandle,
        username: String,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        // TODO: validate name

        let conn = self.pool.get_conn().await;

        if let Err(e) = conn {
            error!("Error getting connection from pool: {}", e);
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let mut conn = conn.unwrap();

        let exists = match account_exists(&mut conn, &username).await {
            Ok(exists) => exists,
            Err(e) => {
                error!("Error checking if account exists: {}", e);
                // Assume it exists if the check fails
                true
            }
        };

        if exists {
            let _ = respond_to.send(Ok(Reply {
                reply_code: AccountReply::Exists,
                data: account::ReplyData::Exists(account::ReplyExists {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let session_id = player.generate_session_id().await;
        let sequence_start = player.get_sequence_start().await;
        let _ = respond_to.send(Ok(Reply {
            reply_code: AccountReply::SessionId(session_id),
            data: account::ReplyData::SessionId(account::ReplySessionId {
                ok: "OK".to_string(),
                sequence_start: sequence_start as EOChar,
            }),
        }));
    }
}
