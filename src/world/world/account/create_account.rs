use eo::protocol::{
    client::account::Create,
    server::account::{Reply, ReplyCreated, ReplyData, ReplyExists},
    AccountReply,
};
use mysql_async::prelude::*;
use tokio::sync::oneshot;

use crate::{errors::WrongSessionIdError, player::PlayerHandle};

use super::super::World;

use super::{account_exists::account_exists, get_password_hash::get_password_hash};

impl World {
    pub async fn create_account(
        &self,
        player: PlayerHandle,
        details: Create,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let session_id = player.take_session_id().await;

        if let Err(e) = session_id {
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let session_id = session_id.unwrap();

        if session_id != details.session_id {
            let _ = respond_to.send(Err(Box::new(WrongSessionIdError::new(
                session_id,
                details.session_id,
            ))));
            return;
        }
        // TODO: validate name

        let conn = self.pool.get_conn().await;

        if let Err(e) = conn {
            error!("Error getting connection from pool: {}", e);
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let mut conn = conn.unwrap();

        let exists = match account_exists(&mut conn, &details.username).await {
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
                data: ReplyData::Exists(ReplyExists {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let password_hash = get_password_hash(&details.username, &details.password);

        let player_ip = player.get_ip_addr().await;
        if let Err(e) = player_ip {
            let _ = respond_to.send(Err(e));
            return;
        }

        let player_ip = player_ip.unwrap();

        match conn
            .exec_drop(
                include_str!("../../../sql/create_account.sql"),
                params! {
                    "name" => &details.username,
                    "password_hash" => &password_hash,
                    "real_name" => &details.fullname,
                    "location" => &details.location,
                    "email" => &details.email,
                    "computer" => &details.computer,
                    "hdid" => &details.hdid,
                    "register_ip" => &player_ip,
                },
            )
            .await
        {
            Ok(_) => {
                info!("New account: {}", details.username);
                let _ = respond_to.send(Ok(Reply {
                    reply_code: AccountReply::Created,
                    data: ReplyData::Created(ReplyCreated {
                        go: "GO".to_string(),
                    }),
                }));
            }
            Err(e) => {
                error!("Error creating account: {}", e);
                let _ = respond_to.send(Err(Box::new(e)));
            }
        }
    }
}
