use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::LoginRequestClientPacket,
        server::{
            LoginReply, LoginReplyServerPacket, LoginReplyServerPacketReplyCodeData,
            LoginReplyServerPacketReplyCodeDataBanned, LoginReplyServerPacketReplyCodeDataBusy,
            LoginReplyServerPacketReplyCodeDataLoggedIn, LoginReplyServerPacketReplyCodeDataOk,
            LoginReplyServerPacketReplyCodeDataWrongUser,
            LoginReplyServerPacketReplyCodeDataWrongUserPassword,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::{params, prelude::Queryable, Params, Row};

use crate::{
    player::{
        player::account::{
            account_banned, account_exists, get_character_list, update_last_login_ip,
            validate_password,
        },
        ClientState,
    },
    SETTINGS,
};

use super::super::Player;

impl Player {
    async fn login_request(&mut self, reader: EoReader) {
        let request = match LoginRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing LoginRequestClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::Accepted {
            self.close("Logging in before connection accepted".to_string())
                .await;
            return;
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

            return;
        }

        let conn = self.pool.get_conn();
        let mut conn = match conn.await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let exists = match account_exists(&mut conn, &request.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        self.login_attempts += 1;

        if !exists {
            if self.login_attempts >= SETTINGS.server.max_login_attempts {
                self.close("Too many login attempts".to_string()).await;
                return;
            }

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
            return;
        }

        let banned = match account_banned(&mut conn, &request.username).await {
            Ok(banned) => banned,
            Err(e) => {
                self.close(format!("Error checking if account is banned: {}", e))
                    .await;
                return;
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
            return;
        }

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../../../sql/get_password_hash.sql"),
                params! {
                    "name" => &request.username,
                },
            )
            .await
        {
            Ok(row) => row,
            Err(e) => {
                self.close(format!("Error getting password hash: {}", e))
                    .await;
                return;
            }
        }
        .unwrap();

        let password_hash: String = row.get("password_hash").unwrap();
        if !validate_password(&request.username, &request.password, &password_hash) {
            if self.login_attempts >= SETTINGS.server.max_login_attempts {
                self.close("Too many login attempts".to_string()).await;
                return;
            }

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
            return;
        }

        let account_id: i32 = row.get("id").unwrap();
        if self.world.is_logged_in(account_id).await {
            if self.login_attempts >= SETTINGS.server.max_login_attempts {
                self.close("Too many login attempts".to_string()).await;
                return;
            }

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
            return;
        }

        if let Err(e) = update_last_login_ip(&mut conn, account_id, &self.ip).await {
            self.close(format!("Error updating last login IP: {}", e))
                .await;
            return;
        }

        let characters = match get_character_list(&mut conn, account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return;
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
    }

    pub async fn handle_login(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.login_request(reader).await,
            _ => error!("Unhandled packet Login_{:?}", action),
        }
    }
}
