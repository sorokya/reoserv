use eo::{
    data::EOChar,
    protocol::{
        client::character::Create,
        server::character::{Reply, ReplyData, ReplyExists, ReplyOk},
        CharacterList, CharacterReply,
    },
};
use mysql_async::{Conn, Row, Params, params, prelude::Queryable};
use tokio::sync::oneshot;
use crate::{character::Character, errors::WrongSessionIdError, player::PlayerHandle};

use super::super::World;

use super::get_character_list::get_character_list;

impl World {
    pub async fn create_character(
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

        let exists = match character_exists(&mut conn, &details.name).await {
            Ok(exists) => exists,
            Err(e) => {
                error!("Error checking if character exists: {}", e);
                // Assume it exists if the check fails
                true
            }
        };

        if exists {
            let _ = respond_to.send(Ok(Reply {
                reply_code: CharacterReply::Exists,
                data: ReplyData::Exists(ReplyExists {
                    no: "NO".to_string(),
                }),
            }));
            return;
        }

        let account_id = match player.get_account_id().await {
            Ok(account_id) => account_id,
            Err(e) => {
                error!("Error getting account_id: {}", e);
                let _ = respond_to.send(Err(Box::new(e)));
                return;
            }
        };

        let mut character = Character::from_creation(account_id, &details);
        if let Err(e) = character.save(&mut conn).await {
            error!(
                "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                e, account_id, details
            );
            let _ = respond_to.send(Err(e));
            return;
        }

        info!("New character: {}", details.name);

        let characters = get_character_list(&mut conn, account_id).await;
        if let Err(e) = characters {
            error!("Error getting character list: {}", e);
            let _ = respond_to.send(Err(e));
            return;
        }

        let characters = characters.unwrap();

        let _ = respond_to.send(Ok(Reply {
            reply_code: CharacterReply::Ok,
            data: ReplyData::Ok(ReplyOk {
                character_list: CharacterList {
                    num_characters: characters.len() as EOChar,
                    characters,
                },
            }),
        }));
    }
}

pub async fn character_exists(
    conn: &mut Conn,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match conn
        .exec_first::<Row, &str, Params>(
            r"SELECT id FROM `Character` WHERE `name` = :name",
            params! {
                "name" => name,
            },
        )
        .await?
    {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}
