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
        return Ok(Reply::no(AccountReply::Exists));
    }

    player.ensure_valid_sequence_for_account_creation().await;
    let sequence_start = player.get_sequence_start().await;

    // TODO: session id
    Ok(Reply::r#continue(1000, sequence_start as EOChar))
}
