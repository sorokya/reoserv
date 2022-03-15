use eo::net::{packets::server::character::Reply, replies::CharacterReply};
use mysql_async::Conn;

use crate::player::PlayerHandle;

use super::get_num_of_characters::get_num_of_characters;

pub async fn request_character_creation(
    conn: &mut Conn,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    let account_id = match player.get_account_id().await {
        Ok(account_id) => account_id,
        Err(e) => {
            error!("Error getting account_id: {}", e);
            return Err(Box::new(e));
        }
    };

    let num_of_characters = get_num_of_characters(conn, account_id).await?;
    // TODO: configurable max number of characters?
    if num_of_characters >= 3 {
        Ok(Reply::no(CharacterReply::Full))
    } else {
        let session_id = player.generate_session_id().await;
        Ok(Reply::r#continue(session_id))
    }
}
