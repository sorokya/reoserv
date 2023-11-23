use eo::{
    data::{EOChar, EOInt, EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::login::{self, Reply},
        CharacterList, LoginReply, PacketAction, PacketFamily,
    },
};
use mysql_async::{prelude::*, Params, Row};

use crate::{player::ClientState, world::WorldHandle};

use super::{super::World, password_hash::validate_password};

use super::{account_exists::account_exists, get_character_list::get_character_list};

impl World {
    pub fn login(
        &mut self,
        player_id: EOShort,
        username: String,
        password: String,
        world: WorldHandle,
    ) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.to_owned(),
            None => return,
        };

        let conn = self.pool.get_conn();
        tokio::spawn(async move {
            let mut conn = match conn.await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            let exists = match account_exists(&mut conn, &username).await {
                Ok(exists) => exists,
                Err(e) => {
                    error!("Error checking if account exists: {}", e);
                    return;
                }
            };

            if !exists {
                let mut builder = StreamBuilder::new();
                builder.add_short(1);
                builder.add_string("NO");
                player.send(PacketAction::Reply, PacketFamily::Login, builder.get());
                return;
            }

            let row = match conn
                .exec_first::<Row, &str, Params>(
                    include_str!("../../../sql/get_password_hash.sql"),
                    params! {
                        "name" => &username,
                    },
                )
                .await
            {
                Ok(row) => row,
                Err(e) => {
                    error!("Error getting password hash: {}", e);
                    let mut builder = StreamBuilder::new();
                    builder.add_short(2);
                    builder.add_string("NO");
                    player.send(PacketAction::Reply, PacketFamily::Login, builder.get());
                    return;
                }
            }
            .unwrap();

            let password_hash: String = row.get("password_hash").unwrap();
            if !validate_password(&username, &password, &password_hash) {
                let mut builder = StreamBuilder::new();
                builder.add_short(2);
                builder.add_string("NO");
                player.send(PacketAction::Reply, PacketFamily::Login, builder.get());
                return;
            }

            let account_id: EOInt = row.get("id").unwrap();
            if world.is_logged_in(account_id).await {
                let mut builder = StreamBuilder::new();
                builder.add_short(5);
                builder.add_string("NO");
                player.send(PacketAction::Reply, PacketFamily::Login, builder.get());
                return;
            }

            let characters = get_character_list(&mut conn, account_id).await;
            if let Err(e) = characters {
                error!("Error getting character list: {}", e);
                return;
            }

            let characters = characters.unwrap();

            world.add_logged_in_account(account_id);
            player.set_account_id(account_id);
            player.set_state(ClientState::LoggedIn);

            let reply = Reply {
                reply_code: LoginReply::Ok,
                data: login::ReplyData::Ok(login::ReplyOk {
                    character_list: CharacterList {
                        num_characters: characters.len() as EOChar,
                        characters,
                    },
                }),
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Login, builder.get());
        });
    }
}
