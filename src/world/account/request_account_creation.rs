use eo::{
    data::EOChar,
    protocol::{
        server::account::{self, Reply},
        AccountReply,
    },
};
use mysql_async::Conn;

use crate::player::PlayerHandle;

use super::account_exists::account_exists;

pub async fn request_account_creation(
    conn: &mut Conn,
    name: String,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    // TODO: validate name

    let exists = account_exists(conn, &name).await?;
    if exists {
        Ok(Reply {
            reply_code: AccountReply::Exists,
            data: account::ReplyData::Exists(account::ReplyExists {
                no: "NO".to_string(),
            }),
        })
    } else {
        let session_id = player.generate_session_id().await;
        player.ensure_valid_sequence_for_account_creation().await;
        let sequence_start = player.get_sequence_start().await;
        Ok(Reply {
            reply_code: AccountReply::SessionId(session_id),
            data: account::ReplyData::SessionId(account::ReplySessionId {
                ok: "OK".to_string(),
                sequence_start: sequence_start as EOChar,
            }),
        })
    }
}
