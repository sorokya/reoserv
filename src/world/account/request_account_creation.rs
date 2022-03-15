use eo::{
    data::EOChar,
    net::{packets::server::account::Reply, replies::AccountReply},
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
        Ok(Reply::no(AccountReply::Exists))
    } else {
        let session_id = player.generate_session_id().await;
        player.ensure_valid_sequence_for_account_creation().await;
        let sequence_start = player.get_sequence_start().await;
        Ok(Reply::r#continue(session_id, sequence_start as EOChar))
    }
}
