use eo::net::{packets::server::account::Reply, replies::AccountReply};
use mysql_async::{prelude::*, Conn};
use sha2::{Digest, Sha256};

use crate::{errors::WrongSessionIdError, player::PlayerHandle, SETTINGS};

use super::account_exists;

pub async fn create_account(
    conn: &mut Conn,
    player: PlayerHandle,
    details: eo::net::packets::client::account::Create,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    match player.take_session_id().await {
        Ok(session_id) => {
            if session_id != details.session_id {
                return Err(Box::new(WrongSessionIdError::new(
                    session_id,
                    details.session_id,
                )));
            }
            // TODO: validate name

            let exists = account_exists(conn, &details.name).await?;
            if exists {
                return Ok(Reply::no(AccountReply::Exists));
            }

            let hash_input = format!(
                "{}{}{}",
                SETTINGS.server.password_salt, details.name, details.password
            );
            let hash = Sha256::digest(hash_input.as_bytes());
            let hash_str = format!("{:x}", hash);

            let player_ip = player.get_ip_addr().await;

            conn.exec_drop(
                include_str!("../../sql/create_account.sql"),
                params! {
                    "name" => &details.name,
                    "password_hash" => &hash_str,
                    "real_name" => &details.fullname,
                    "location" => &details.location,
                    "email" => &details.email,
                    "computer" => &details.computer,
                    "hdid" => &details.hdid,
                    "register_ip" => &player_ip,
                },
            )
            .await?;
            info!("New account: {}", details.name);
            Ok(Reply::ok(AccountReply::Created))
        }
        Err(e) => Err(Box::new(e)),
    }
}
