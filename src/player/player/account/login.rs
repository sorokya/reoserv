
use eolib::protocol::net::server::{
    LoginReply, LoginReplyServerPacket, LoginReplyServerPacketReplyCodeData,
    LoginReplyServerPacketReplyCodeDataBanned, LoginReplyServerPacketReplyCodeDataBusy,
    LoginReplyServerPacketReplyCodeDataLoggedIn, LoginReplyServerPacketReplyCodeDataOk,
    LoginReplyServerPacketReplyCodeDataWrongUser,
    LoginReplyServerPacketReplyCodeDataWrongUserPassword,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use mysql_async::{prelude::*, Params, Row};

use crate::player::ClientState;
use crate::SETTINGS;

use super::{
    super::Player, account_banned::account_banned, password_hash::validate_password,
    update_last_login_ip::update_last_login_ip,
};

use super::{account_exists::account_exists, get_character_list::get_character_list};

impl Player {
    pub async fn login(&mut self, username: String, password: String) -> bool {
        if self.state != ClientState::Accepted {
            return true;
        }

        let player_count = self.world.get_player_count().await;
        if player_count >= SETTINGS.server.max_players {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::Busy,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::Busy(
                            LoginReplyServerPacketReplyCodeDataBusy::new(),
                        )),
                    },
                )
                .await;

            self.close("Server busy".to_string()).await;

            return false;
        }

        let conn = self.pool.get_conn();
        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return false;
            }
        };

        let exists = match account_exists(&mut conn, &username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return false;
            }
        };

        if !exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::WrongUser,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::WrongUser(
                            LoginReplyServerPacketReplyCodeDataWrongUser::new(),
                        )),
                    },
                )
                .await;
            return true;
        }

        let banned = match account_banned(&mut conn, &username).await {
            Ok(banned) => banned,
            Err(e) => {
                self.close(format!("Error checking if account is banned: {}", e))
                    .await;
                return false;
            }
        };

        if banned {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::Banned,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::Banned(
                            LoginReplyServerPacketReplyCodeDataBanned::new(),
                        )),
                    },
                )
                .await;
            self.close("Account is banned".to_string()).await;
            return false;
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

                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Login,
                        LoginReplyServerPacket {
                            reply_code: LoginReply::WrongUserPassword,
                            reply_code_data: Some(
                                LoginReplyServerPacketReplyCodeData::WrongUserPassword(
                                    LoginReplyServerPacketReplyCodeDataWrongUserPassword::new(),
                                ),
                            ),
                        },
                    )
                    .await;
                return true;
            }
        }
        .unwrap();

        let password_hash: String = row.get("password_hash").unwrap();
        if !validate_password(&username, &password, &password_hash) {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::WrongUserPassword,
                        reply_code_data: Some(
                            LoginReplyServerPacketReplyCodeData::WrongUserPassword(
                                LoginReplyServerPacketReplyCodeDataWrongUserPassword::new(),
                            ),
                        ),
                    },
                )
                .await;
            return true;
        }

        let account_id: i32 = row.get("id").unwrap();
        if self.world.is_logged_in(account_id).await {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Login,
                    LoginReplyServerPacket {
                        reply_code: LoginReply::LoggedIn,
                        reply_code_data: Some(LoginReplyServerPacketReplyCodeData::LoggedIn(
                            LoginReplyServerPacketReplyCodeDataLoggedIn::new(),
                        )),
                    },
                )
                .await;
            return true;
        }

        if let Err(e) = update_last_login_ip(&mut conn, account_id, &self.ip).await {
            self.close(format!("Error updating last login IP: {}", e))
                .await;
            return false;
        }

        let characters = match get_character_list(&mut conn, account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return false;
            }
        };

        self.world.add_logged_in_account(account_id);
        self.account_id = account_id;
        self.state = ClientState::LoggedIn;

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Login,
                LoginReplyServerPacket {
                    reply_code: LoginReply::OK,
                    reply_code_data: Some(LoginReplyServerPacketReplyCodeData::OK(
                        LoginReplyServerPacketReplyCodeDataOk { characters },
                    )),
                },
            )
            .await;

        true
    }
}
