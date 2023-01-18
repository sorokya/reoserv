use eo::{
    data::EOChar,
    protocol::{
        client::character::Create,
        server::character::{Reply, ReplyData, ReplyExists, ReplyOk},
        CharacterList, CharacterReply,
    },
};
use mysql_async::Conn;

use crate::{character::Character, errors::WrongSessionIdError, player::PlayerHandle};

use super::{character_exists, get_character_list};

pub async fn create_character(
    conn: &mut Conn,
    details: Create,
    player: PlayerHandle,
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

            if character_exists(conn, &details.name).await? {
                return Ok(Reply {
                    reply_code: CharacterReply::Exists,
                    data: ReplyData::Exists(ReplyExists {
                        no: "NO".to_string(),
                    }),
                });
            }

            let account_id = match player.get_account_id().await {
                Ok(account_id) => account_id,
                Err(e) => {
                    error!("Error getting account_id: {}", e);
                    return Err(Box::new(e));
                }
            };

            let mut character = Character::from_creation(account_id, &details);
            if let Err(e) = character.save(conn).await {
                error!(
                    "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                    e, account_id, details
                );
                return Err(e);
            }

            info!("New character: {}", details.name);

            let characters = get_character_list(conn, account_id).await?;

            Ok(Reply {
                reply_code: CharacterReply::Ok,
                data: ReplyData::Ok(ReplyOk {
                    character_list: CharacterList {
                        num_characters: characters.len() as EOChar,
                        characters,
                    },
                }),
            })
        }
        Err(e) => Err(Box::new(e)),
    }
}
