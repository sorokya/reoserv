use crate::{character::Character, errors::WrongSessionIdError};
use eo::{
    data::{i32, i32, Serializeable, StreamBuilder},
    protocol::{
        client::character::Create,
        server::character::{Reply, ReplyData, ReplyExists, ReplyOk},
        CharacterList, CharacterReply, PacketAction, PacketFamily,
    },
};
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

use super::super::World;

use super::get_character_list::get_character_list;

impl World {
    pub async fn create_character(&self, player_id: i32, details: Create) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.clone(),
            None => return,
        };

        let conn = self.pool.get_conn();

        tokio::spawn(async move {
            let mut conn = match conn.await {
                Ok(conn) => conn,
                Err(e) => {
                    player.close(format!("Error getting connection from pool: {}", e));
                    return;
                }
            };

            let session_id = player.take_session_id().await;
            if let Err(e) = session_id {
                player.close(format!("Error getting session id: {}", e));
                return;
            }

            let session_id = session_id.unwrap();

            if session_id != details.session_id {
                player.close(format!(
                    "{}",
                    WrongSessionIdError::new(session_id, details.session_id)
                ));
                return;
            }

            // TODO: validate name

            let exists = match character_exists(&mut conn, &details.name).await {
                Ok(exists) => exists,
                Err(e) => {
                    player.close(format!("Error checking if character exists: {}", e));
                    // Assume it exists if the check fails
                    true
                }
            };

            if exists {
                let reply = Reply {
                    reply_code: CharacterReply::Exists,
                    data: ReplyData::Exists(ReplyExists {
                        no: "NO".to_string(),
                    }),
                };

                let mut builder = StreamBuilder::new();
                reply.serialize(&mut builder);
                player.send(PacketAction::Reply, PacketFamily::Character, builder.get());

                return;
            }

            let account_id = match player.get_account_id().await {
                Ok(account_id) => account_id,
                Err(e) => {
                    player.close(format!("Error getting account_id: {}", e));
                    return;
                }
            };

            let mut character = Character::from_creation(account_id, &details);
            if let Err(e) = character.save(&mut conn).await {
                player.close(format!(
                    "Error creating character: {}\n\taccount_id: {}\n\tdetails: {:?}",
                    e, account_id, details
                ));
                return;
            }

            info!("New character: {}", details.name);

            let characters = get_character_list(&mut conn, account_id).await;
            if let Err(e) = characters {
                player.close(format!("Error getting character list: {}", e));
                return;
            }

            let characters = characters.unwrap();

            let reply = Reply {
                reply_code: CharacterReply::Ok,
                data: ReplyData::Ok(ReplyOk {
                    character_list: CharacterList {
                        num_characters: characters.len() as i32,
                        characters,
                    },
                }),
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Character, builder.get());
        });
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
