use eolib::{
    data::{EoReader, EoSerialize, EoWriter},
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
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

use crate::{
    deep::{
        AccountRecoverPinReply, AccountRecoverReply, AccountRecoverUpdateReply,
        LoginAcceptClientPacket, LoginAcceptServerPacket, LoginAgreeClientPacket,
        LoginAgreeServerPacket, LoginConfigServerPacket, LoginCreateClientPacket,
        LoginCreateServerPacket, LoginTakeClientPacket, LoginTakeServerPacket, ACTION_CONFIG,
    },
    player::{
        player::account::{
            account_banned, account_exists, generate_password_hash, generate_session,
            get_character_list, update_last_login_ip, validate_password,
        },
        ClientState,
    },
    utils::{is_deep, mask_email, send_email},
    EMAILS, SETTINGS,
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

        let remember_me = if reader.remaining() > 0 {
            reader.get_char() == 1
        } else {
            false
        };

        if self.state != ClientState::Accepted {
            self.close("Logging in before connection accepted".to_string())
                .await;
            return;
        }

        let player_count = self.world.get_player_count().await.expect("Failed to get player count. Timeout");
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

        let username: String = row.get("name").unwrap();
        let password_hash: String = row.get("password_hash").unwrap();
        let account_id: i32 = row.get("id").unwrap();
        let logged_in = self.world.is_logged_in(account_id).await.expect("Failed to check if logged in. Timeout");
        self.world.add_pending_login(account_id);

        if !validate_password(&username, &request.password, &password_hash) {
            self.world.remove_pending_login(account_id);
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

        if logged_in {
            self.world.remove_pending_login(account_id);
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

        self.finish_login(&mut conn, account_id, remember_me).await;
    }

    async fn finish_login(&mut self, conn: &mut Conn, account_id: i32, remember_me: bool) {
        let session_token = if remember_me {
            match generate_session(conn, account_id).await {
                Ok(token) => Some(token),
                Err(e) => {
                    self.world.remove_pending_login(account_id);
                    self.close(format!("Error generating session: {}", e)).await;
                    return;
                }
            }
        } else {
            None
        };

        if let Err(e) = update_last_login_ip(conn, account_id, &self.ip).await {
            self.world.remove_pending_login(account_id);
            self.close(format!("Error updating last login IP: {}", e))
                .await;
            return;
        }

        let characters = match get_character_list(conn, account_id).await {
            Ok(characters) => characters,
            Err(e) => {
                self.close(format!("Error getting character list: {}", e))
                    .await;
                return;
            }
        };

        self.account_id = account_id;
        self.world.add_logged_in_account(account_id);
        self.state = ClientState::LoggedIn;

        if is_deep(&self.version) {
            let _ = self
                .bus
                .send(
                    PacketAction::Unrecognized(ACTION_CONFIG),
                    PacketFamily::Login,
                    LoginConfigServerPacket {
                        max_skins: SETTINGS.character.max_skin + 1,
                        max_hair_modals: SETTINGS.character.max_hair_style,
                        max_character_name: SETTINGS.character.max_name_length as i32,
                    },
                )
                .await;
        }

        let packet = LoginReplyServerPacket {
            reply_code: LoginReply::OK,
            reply_code_data: Some(LoginReplyServerPacketReplyCodeData::OK(
                LoginReplyServerPacketReplyCodeDataOk { characters },
            )),
        };

        let mut writer = EoWriter::new();
        if let Err(e) = packet.serialize(&mut writer) {
            self.world.remove_pending_login(account_id);
            self.close(format!("Failed to serialize LoginReplyServerPacket: {}", e))
                .await;
            return;
        }

        if let Some(token) = session_token {
            writer.add_string(&token);
        }

        let _ = self
            .bus
            .send_buf(
                PacketAction::Reply,
                PacketFamily::Login,
                writer.to_byte_array(),
            )
            .await;
    }

    async fn login_take(&mut self, reader: EoReader) {
        if let Err(e) = LoginTakeClientPacket::deserialize(&reader) {
            error!("Failed to deserialize LoginTakeClientPacket: {}", e);
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Take,
                PacketFamily::Login,
                LoginTakeServerPacket {
                    reply_code: if SETTINGS.account.recovery {
                        AccountRecoverReply::RequestAccepted
                    } else {
                        AccountRecoverReply::RecoveryDisabled
                    },
                },
            )
            .await;
    }

    async fn login_create(&mut self, reader: EoReader) {
        let create = match LoginCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                error!("Failed to deserialize LoginCreateClientPacket: {}", e);
                return;
            }
        };

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get SQL connection: {}", e);
                return;
            }
        };

        let mut row: Row = match conn
            .exec_first(
                include_str!("../../../sql/get_account_email.sql"),
                params! {
                    "name" => &create.account_name,
                },
            )
            .await
        {
            Ok(Some(row)) => row,
            Ok(None) => {
                let _ = self
                    .bus
                    .send(
                        PacketAction::Create,
                        PacketFamily::Login,
                        LoginCreateServerPacket {
                            reply_code: AccountRecoverReply::AccountNotFound,
                            email_address: None,
                        },
                    )
                    .await;
                return;
            }
            Err(e) => {
                error!("Failed to get account email: {}", e);
                return;
            }
        };

        self.account_id = match row.take(0) {
            Some(id) => id,
            None => return,
        };

        let email: String = match row.take(1) {
            Some(email) => email,
            None => return,
        };

        let code = self.generate_email_pin();

        if let Err(e) = send_email(
            &email,
            &create.account_name,
            &get_lang_string!(&EMAILS.recovery.subject, name = create.account_name),
            &get_lang_string!(
                &EMAILS.recovery.body,
                name = create.account_name,
                code = code
            ),
        )
        .await
        {
            self.close(format!("Failed to send recovery email: {}", e))
                .await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Create,
                PacketFamily::Login,
                LoginCreateServerPacket {
                    reply_code: if SETTINGS.account.recovery_show_email {
                        AccountRecoverReply::RequestAcceptedShowEmail
                    } else {
                        AccountRecoverReply::RequestAccepted
                    },
                    email_address: if SETTINGS.account.recovery_show_email {
                        Some(if SETTINGS.account.recovery_mask_email {
                            mask_email(&email)
                        } else {
                            email
                        })
                    } else {
                        None
                    },
                },
            )
            .await;
    }

    async fn login_accept(&mut self, reader: EoReader) {
        let accept = match LoginAcceptClientPacket::deserialize(&reader) {
            Ok(accept) => accept,
            Err(e) => {
                error!("Failed to deserialize LoginAcceptClientPacket: {}", e);
                return;
            }
        };

        let pin = match &self.email_pin {
            Some(pin) => pin,
            None => return,
        };

        let _ = self
            .bus
            .send(
                PacketAction::Accept,
                PacketFamily::Login,
                LoginAcceptServerPacket {
                    reply_code: if *pin != accept.pin {
                        AccountRecoverPinReply::WrongPin
                    } else {
                        AccountRecoverPinReply::OK
                    },
                },
            )
            .await;
    }

    async fn login_agree(&mut self, reader: EoReader) {
        let agree = match LoginAgreeClientPacket::deserialize(&reader) {
            Ok(agree) => agree,
            Err(e) => {
                error!("Failed to deserialize LoginAgreeClientPacket: {}", e);
                self.send_login_agree_error().await;
                return;
            }
        };

        if self.account_id == 0 {
            self.send_login_agree_error().await;
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Failed to get SQL connection: {}", e);
                self.send_login_agree_error().await;
                return;
            }
        };

        let password_hash = generate_password_hash(&agree.account_name, &agree.password);

        if let Err(e) = conn
            .exec_drop(
                include_str!("../../../sql/update_password_hash.sql"),
                params! {
                    "id" => self.account_id,
                    "password_hash" => &password_hash,
                },
            )
            .await
        {
            error!("Error updating password hash: {}", e);
            self.send_login_agree_error().await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Agree,
                PacketFamily::Login,
                LoginAgreeServerPacket {
                    reply_code: AccountRecoverUpdateReply::OK,
                },
            )
            .await;
    }

    async fn send_login_agree_error(&mut self) {
        let _ = self
            .bus
            .send(
                PacketAction::Agree,
                PacketFamily::Login,
                LoginAgreeServerPacket {
                    reply_code: AccountRecoverUpdateReply::Error,
                },
            )
            .await;
    }

    async fn login_use(&mut self, reader: EoReader) {
        let token = reader.get_string();
        if token.is_empty() {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return;
            }
        };

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../../../sql/get_session.sql"),
                params! {
                    "token" => &token,
                },
            )
            .await
        {
            Ok(row) => row,
            Err(e) => {
                self.close(format!("Error getting session: {}", e)).await;
                return;
            }
        };

        match row {
            Some(row) => {
                let account_id: i32 = row.get("account_id").unwrap();
                let logged_in = self.world.is_logged_in(account_id).await.expect("Failed to check if logged in. Timeout");
                if logged_in {
                    self.world.remove_pending_login(account_id);
                    let _ = self
                        .bus
                        .send(
                            PacketAction::Reply,
                            PacketFamily::Login,
                            LoginReplyServerPacket {
                                reply_code: LoginReply::LoggedIn,
                                reply_code_data: Some(
                                    LoginReplyServerPacketReplyCodeData::LoggedIn(
                                        LoginReplyServerPacketReplyCodeDataLoggedIn::new(),
                                    ),
                                ),
                            },
                        )
                        .await;
                    return;
                }
                self.finish_login(&mut conn, account_id, true).await;
            }
            None => {
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
            }
        }
    }

    pub async fn handle_login(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Request => self.login_request(reader).await,
            PacketAction::Take => self.login_take(reader).await,
            PacketAction::Create => self.login_create(reader).await,
            PacketAction::Accept => self.login_accept(reader).await,
            PacketAction::Agree => self.login_agree(reader).await,
            PacketAction::Use => self.login_use(reader).await,
            _ => error!("Unhandled packet Login_{:?}", action),
        }
    }
}
