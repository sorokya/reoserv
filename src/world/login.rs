use eo::{data::{EOShort, EOChar}, net::CharacterList};
use mysql_async::{prelude::*, Conn, Params, Row};
use tokio::sync::MutexGuard;

use super::{LoginResult, account_exists::account_exists, get_character_list::get_character_list};

pub async fn login(
    conn: &mut Conn,
    name: &str,
    password_hash: &str,
    accounts: &mut MutexGuard<'_, Vec<EOShort>>,
) -> LoginResult {
    let exists = match account_exists(conn, &name).await {
        Ok(exists) => exists,
        Err(e) => {
            error!("Error checking if account exists: {}", e);
            return LoginResult::Err(e);
        },
    };

    if !exists {
        return LoginResult::WrongUsername;
    }

    match conn.exec_first::<Row, &str, Params>(
        include_str!("../sql/verify_password.sql"),
        params! {
            "name" => &name,
            "password_hash" => &password_hash,
        },
    ).await {
        Ok(row) => match row {
            Some(row) => {
                let account_id: EOShort = row.get("id").unwrap();
                if accounts.contains(&account_id) {
                    return LoginResult::LoggedIn;
                }

                let characters = match get_character_list(conn, account_id).await {
                    Ok(characters) => characters,
                    Err(e) => {
                        error!("Error getting character list: {}", e);
                        return LoginResult::Err(e);
                    }
                };
                accounts.push(account_id);
                let character_list = CharacterList {
                    length: characters.len() as EOChar,
                    unknown: 1,
                    characters,
                };
                LoginResult::Success { account_id, character_list }
            },
            None => LoginResult::WrongPassword,
        },
        Err(e) => {
            error!("Error verifying password: {}", e);
            LoginResult::Err(Box::new(e))
        },
    }
}
