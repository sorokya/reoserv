use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::server::{
    LoginReply, LoginReplyServerPacket, LoginReplyServerPacketReplyCodeData,
    LoginReplyServerPacketReplyCodeDataBanned, LoginReplyServerPacketReplyCodeDataLoggedIn,
    LoginReplyServerPacketReplyCodeDataOk, LoginReplyServerPacketReplyCodeDataWrongUser,
    LoginReplyServerPacketReplyCodeDataWrongUserPassword,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use mysql_async::{prelude::*, Params, Row};

use crate::{player::ClientState, world::WorldHandle};

use super::{
    super::World, account_banned::account_banned, password_hash::validate_password,
    update_last_login_ip::update_last_login_ip,
};

use super::{account_exists::account_exists, get_character_list::get_character_list};

impl World {
    pub fn login(
        &mut self,
        player_id: i32,
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
                let packet = LoginReplyServerPacket {
                    reply_code: LoginReply::WrongUser,
                    reply_code_data: Some(LoginReplyServerPacketReplyCodeData::WrongUser(
                        LoginReplyServerPacketReplyCodeDataWrongUser::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize LoginReplyServerPacket: {}", e);
                    return;
                }

                player.send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    writer.to_byte_array(),
                );
                return;
            }

            let banned = match account_banned(&mut conn, &username).await {
                Ok(banned) => banned,
                Err(e) => {
                    error!("Error checking if account is banned: {}", e);
                    return;
                }
            };

            if banned {
                let packet = LoginReplyServerPacket {
                    reply_code: LoginReply::Banned,
                    reply_code_data: Some(LoginReplyServerPacketReplyCodeData::Banned(
                        LoginReplyServerPacketReplyCodeDataBanned::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize LoginReplyServerPacket: {}", e);
                    return;
                }

                player.send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    writer.to_byte_array(),
                );
                player.close("Account is banned".to_string());
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

                    let packet = LoginReplyServerPacket {
                        reply_code: LoginReply::WrongUserPassword,
                        reply_code_data: Some(
                            LoginReplyServerPacketReplyCodeData::WrongUserPassword(
                                LoginReplyServerPacketReplyCodeDataWrongUserPassword::new(),
                            ),
                        ),
                    };

                    let mut writer = EoWriter::new();

                    if let Err(e) = packet.serialize(&mut writer) {
                        error!("Failed to serialize LoginReplyServerPacket: {}", e);
                        return;
                    }

                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Login,
                        writer.to_byte_array(),
                    );
                    return;
                }
            }
            .unwrap();

            let password_hash: String = row.get("password_hash").unwrap();
            if !validate_password(&username, &password, &password_hash) {
                let packet = LoginReplyServerPacket {
                    reply_code: LoginReply::WrongUserPassword,
                    reply_code_data: Some(LoginReplyServerPacketReplyCodeData::WrongUserPassword(
                        LoginReplyServerPacketReplyCodeDataWrongUserPassword::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize LoginReplyServerPacket: {}", e);
                    return;
                }

                player.send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    writer.to_byte_array(),
                );
                return;
            }

            let account_id: i32 = row.get("id").unwrap();
            if world.is_logged_in(account_id).await {
                let packet = LoginReplyServerPacket {
                    reply_code: LoginReply::LoggedIn,
                    reply_code_data: Some(LoginReplyServerPacketReplyCodeData::LoggedIn(
                        LoginReplyServerPacketReplyCodeDataLoggedIn::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize LoginReplyServerPacket: {}", e);
                    return;
                }

                player.send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    writer.to_byte_array(),
                );
                return;
            }

            let player_ip = match player.get_ip_addr().await {
                Ok(ip) => ip,
                Err(e) => {
                    player.close(format!("Error getting player IP: {}", e));
                    return;
                }
            };

            if let Err(e) = update_last_login_ip(&mut conn, account_id, &player_ip).await {
                player.close(format!("Error updating last login IP: {}", e));
                return;
            }

            let characters = match get_character_list(&mut conn, account_id).await {
                Ok(characters) => characters,
                Err(e) => {
                    player.close(format!("Error getting character list: {}", e));
                    return;
                }
            };

            world.add_logged_in_account(account_id);
            player.set_account_id(account_id);
            player.set_state(ClientState::LoggedIn);

            let reply = LoginReplyServerPacket {
                reply_code: LoginReply::OK,
                reply_code_data: Some(LoginReplyServerPacketReplyCodeData::OK(
                    LoginReplyServerPacketReplyCodeDataOk { characters },
                )),
            };

            let mut writer = EoWriter::new();
            if let Err(e) = reply.serialize(&mut writer) {
                error!("Failed to serialize LoginReplyServerPacket: {}", e);
                return;
            }

            player.send(
                PacketAction::Reply,
                PacketFamily::Login,
                writer.to_byte_array(),
            );
        });
    }
}
