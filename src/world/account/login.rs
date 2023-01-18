use eo::{
    data::{EOChar, EOInt},
    protocol::{
        server::login::{self, Reply},
        CharacterList, LoginReply,
    },
};
use mysql_async::{prelude::*, Conn, Params, Row};
use sha2::{Digest, Sha256};

use crate::SETTINGS;

use super::{account_exists::account_exists, get_character_list::get_character_list};

pub async fn login(
    conn: &mut Conn,
    name: &str,
    password: &str,
    accounts: &mut Vec<EOInt>,
) -> Result<(Reply, EOInt), Box<dyn std::error::Error + Send + Sync>> {
    let exists = account_exists(conn, name).await?;
    if !exists {
        return Ok((
            Reply {
                reply_code: LoginReply::WrongUser,
                data: login::ReplyData::WrongUser(login::ReplyWrongUser {
                    no: "NO".to_string(),
                }),
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
                    reply_code: LoginReply::WrongUserPass,
                    data: login::ReplyData::WrongUserPass(login::ReplyWrongUserPass {
                        no: "NO".to_string(),
                    }),
                },
                0,
            ))
        }
    };

    let account_id: EOInt = row.get("id").unwrap();
    if accounts.contains(&account_id) {
        return Ok((
            Reply {
                reply_code: LoginReply::LoggedIn,
                data: login::ReplyData::LoggedIn(login::ReplyLoggedIn {
                    no: "NO".to_string(),
                }),
            },
            0,
        ));
    }

    let characters = get_character_list(conn, account_id).await?;
    accounts.push(account_id);
    Ok((
        Reply {
            reply_code: LoginReply::Ok,
            data: login::ReplyData::Ok(login::ReplyOk {
                character_list: CharacterList {
                    num_characters: characters.len() as EOChar,
                    characters,
                },
            }),
        },
        account_id,
    ))
}
