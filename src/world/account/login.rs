use eo::{
    data::{EOChar, EOInt},
    net::{packets::server::login::Reply, replies::LoginReply, CharacterList},
};
use mysql_async::{prelude::*, Conn, Params, Row};
use sha2::{Digest, Sha256};
use tokio::sync::MutexGuard;

use crate::SETTINGS;

use super::{account_exists::account_exists, get_character_list::get_character_list};

pub async fn login(
    conn: &mut Conn,
    name: &str,
    password: &str,
    accounts: &mut MutexGuard<'_, Vec<EOInt>>,
) -> Result<(Reply, EOInt), Box<dyn std::error::Error + Send + Sync>> {
    let exists = account_exists(conn, name).await?;
    if !exists {
        return Ok((
            Reply {
                reply: LoginReply::WrongUsername,
                character_list: None,
            },
            0,
        ));
    }

    let hash_input = format!("{}{}{}", SETTINGS.server.password_salt, name, password);
    let hash = Sha256::digest(hash_input.as_bytes());
    let hash_str = format!("{:x}", hash);

    let row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../../sql/verify_password.sql"),
            params! {
                "name" => &name,
                "password_hash" => &hash_str,
            },
        )
        .await?
    {
        Some(row) => row,
        None => {
            return Ok((
                Reply {
                    reply: LoginReply::WrongPassword,
                    character_list: None,
                },
                0,
            ))
        }
    };

    let account_id: EOInt = row.get("id").unwrap();
    if accounts.contains(&account_id) {
        return Ok((
            Reply {
                reply: LoginReply::LoggedIn,
                character_list: None,
            },
            0,
        ));
    }

    let characters = get_character_list(conn, account_id).await?;
    accounts.push(account_id);
    Ok((
        Reply {
            reply: LoginReply::OK,
            character_list: Some(CharacterList {
                length: characters.len() as EOChar,
                unknown: 1,
                characters,
            }),
        },
        account_id,
    ))
}
