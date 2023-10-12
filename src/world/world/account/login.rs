use eo::{
    data::{EOChar, EOInt},
    protocol::{
        server::login::{self, Reply},
        CharacterList, LoginReply,
    },
};
use mysql_async::{prelude::*, Params, Row};
use tokio::sync::oneshot;

use crate::player::{ClientState, PlayerHandle};

use super::{super::World, password_hash::validate_password};

use super::{account_exists::account_exists, get_character_list::get_character_list};

impl World {
    pub async fn login(
        &mut self,
        player: PlayerHandle,
        username: &str,
        password: &str,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let conn = self.pool.get_conn().await;

        if let Err(e) = conn {
            error!("Error getting connection from pool: {}", e);
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let mut conn = conn.unwrap();

        let exists = match account_exists(&mut conn, username).await {
            Ok(exists) => exists,
            Err(e) => {
                error!("Error checking if account exists: {}", e);
                // Assume it exists if the check fails
                true
            }
        };

        if !exists {
            let _ = respond_to.send(Ok(Reply {
                reply_code: LoginReply::WrongUser,
                data: login::ReplyData::WrongUser(login::ReplyWrongUser {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../../../sql/get_password_hash.sql"),
                params! {
                    "name" => username,
                },
            )
            .await
        {
            Ok(row) => row,
            Err(e) => {
                error!("Error getting password hash: {}", e);
                let _ = respond_to.send(Err(Box::new(e)));
                return;
            }
        }
        .unwrap();

        let password_hash: String = row.get("password_hash").unwrap();
        if !validate_password(username, password, &password_hash) {
            let _ = respond_to.send(Ok(Reply {
                reply_code: LoginReply::WrongUserPass,
                data: login::ReplyData::WrongUserPass(login::ReplyWrongUserPass {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let account_id: EOInt = row.get("id").unwrap();
        if self.accounts.contains(&account_id) {
            let _ = respond_to.send(Ok(Reply {
                reply_code: LoginReply::LoggedIn,
                data: login::ReplyData::LoggedIn(login::ReplyLoggedIn {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let characters = get_character_list(&mut conn, account_id).await;
        if let Err(e) = characters {
            error!("Error getting character list: {}", e);
            let _ = respond_to.send(Err(e));
            return;
        }

        let characters = characters.unwrap();

        self.accounts.push(account_id);
        player.set_account_id(account_id);
        player.set_state(ClientState::LoggedIn);
        let _ = respond_to.send(Ok(Reply {
            reply_code: LoginReply::Ok,
            data: login::ReplyData::Ok(login::ReplyOk {
                character_list: CharacterList {
                    num_characters: characters.len() as EOChar,
                    characters,
                },
            }),
        }));
    }
}
