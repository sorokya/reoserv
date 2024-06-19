use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{AccountAgreeClientPacket, AccountCreateClientPacket, AccountRequestClientPacket},
        server::{
            AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
            AccountReplyServerPacketReplyCodeDataChangeFailed,
            AccountReplyServerPacketReplyCodeDataChanged,
            AccountReplyServerPacketReplyCodeDataCreated,
            AccountReplyServerPacketReplyCodeDataDefault,
            AccountReplyServerPacketReplyCodeDataExists,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::{params, prelude::Queryable, Params, Row};

use crate::{
    deep::{
        AccountAcceptClientPacket, AccountAcceptServerPacket, AccountConfigServerPacket,
        ACCOUNT_REPLY_WRONG_PIN, ACTION_CONFIG,
    },
    errors::WrongSessionIdError,
    player::{player::account::account_exists, ClientState},
    utils::{generate_password_hash, is_deep, send_email, validate_password},
    EMAILS, SETTINGS,
};

use super::super::Player;

impl Player {
    async fn account_create(&mut self, reader: EoReader) {
        let create = match AccountCreateClientPacket::deserialize(&reader) {
            Ok(create) => create,
            Err(e) => {
                error!("Error deserializing AccountCreateClientPacket {}", e);
                return;
            }
        };

        if let Some(email_pin) = &self.email_pin {
            reader.set_chunked_reading_mode(true);
            let pin = reader.get_string();
            if pin != *email_pin {
                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        AccountReplyServerPacket {
                            reply_code: AccountReply::Unrecognized(ACCOUNT_REPLY_WRONG_PIN),
                            reply_code_data: None,
                        },
                    )
                    .await;
                return;
            }
        }

        if self.state != ClientState::Accepted {
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

        let session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return;
            }
        };

        if session_id != create.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(session_id, create.session_id)
            ))
            .await;
            return;
        }

        // TODO: validate name

        let exists = match account_exists(&mut conn, &create.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        if exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::Exists,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                            AccountReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let password_hash = generate_password_hash(&create.username, &create.password);

        match conn
            .exec_drop(
                include_str!("../../../sql/create_account.sql"),
                params! {
                    "name" => &create.username,
                    "password_hash" => &password_hash,
                    "real_name" => &create.full_name,
                    "location" => &create.location,
                    "email" => &create.email,
                    "computer" => &create.computer,
                    "hdid" => &create.hdid,
                    "register_ip" => &self.ip,
                },
            )
            .await
        {
            Ok(_) => {
                info!("New account: {}", create.username);

                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        AccountReplyServerPacket {
                            reply_code: AccountReply::Created,
                            reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Created(
                                AccountReplyServerPacketReplyCodeDataCreated::new(),
                            )),
                        },
                    )
                    .await;
            }
            Err(e) => {
                self.close(format!("Error creating account: {}", e)).await;
            }
        }
    }

    async fn account_request(&mut self, reader: EoReader) {
        let request = match AccountRequestClientPacket::deserialize(&reader) {
            Ok(request) => request,
            Err(e) => {
                error!("Error deserializing AccountRequestClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::Accepted {
            return;
        }

        // TODO: validate name

        let mut conn = match self.pool.get_conn().await {
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

        if exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::Exists,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                            AccountReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        if is_deep(&self.version) {
            let _ = self
                .bus
                .send(
                    PacketAction::Unrecognized(ACTION_CONFIG),
                    PacketFamily::Account,
                    AccountConfigServerPacket {
                        delay_time: SETTINGS.account.delay_time,
                        email_validation: SETTINGS.account.email_validation,
                    },
                )
                .await;
        }

        let session_id = self.generate_session_id();
        let sequence_start = self.bus.sequencer.get_start();

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Account,
                AccountReplyServerPacket {
                    reply_code: AccountReply::Unrecognized(session_id),
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Default(
                        AccountReplyServerPacketReplyCodeDataDefault { sequence_start },
                    )),
                },
            )
            .await;
    }

    async fn account_agree(&mut self, reader: EoReader) {
        let agree = match AccountAgreeClientPacket::deserialize(&reader) {
            Ok(agree) => agree,
            Err(e) => {
                error!("Error deserializing AccountAgreeClientPacket {}", e);
                return;
            }
        };

        if self.state != ClientState::LoggedIn {
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

        let exists = match account_exists(&mut conn, &agree.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return;
            }
        };

        if !exists {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::Exists,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                            AccountReplyServerPacketReplyCodeDataExists::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let row = match conn
            .exec_first::<Row, &str, Params>(
                include_str!("../../../sql/get_password_hash.sql"),
                params! {
                    "name" => &agree.username,
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
                        PacketFamily::Account,
                        AccountReplyServerPacket {
                            reply_code: AccountReply::ChangeFailed,
                            reply_code_data: Some(
                                AccountReplyServerPacketReplyCodeData::ChangeFailed(
                                    AccountReplyServerPacketReplyCodeDataChangeFailed::new(),
                                ),
                            ),
                        },
                    )
                    .await;
                return;
            }
        }
        .unwrap();

        let username: String = row.get("name").unwrap();
        let password_hash: String = row.get("password_hash").unwrap();
        if !validate_password(&username, &agree.old_password, &password_hash) {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    AccountReplyServerPacket {
                        reply_code: AccountReply::ChangeFailed,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::ChangeFailed(
                            AccountReplyServerPacketReplyCodeDataChangeFailed::new(),
                        )),
                    },
                )
                .await;
            return;
        }

        let account_id: i32 = row.get("id").unwrap();

        let password_hash = generate_password_hash(&username, &agree.new_password);
        if let Err(e) = conn
            .exec_drop(
                include_str!("../../../sql/update_password_hash.sql"),
                params! {
                    "id" => account_id,
                    "password_hash" => &password_hash,
                },
            )
            .await
        {
            self.close(format!("Error updating password hash: {}", e))
                .await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Account,
                AccountReplyServerPacket {
                    reply_code: AccountReply::Changed,
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Changed(
                        AccountReplyServerPacketReplyCodeDataChanged::new(),
                    )),
                },
            )
            .await;
    }

    async fn account_accept(&mut self, reader: EoReader) {
        let accept = match AccountAcceptClientPacket::deserialize(&reader) {
            Ok(accept) => accept,
            Err(e) => {
                error!("Faled to deserialize AccountAcceptClientPacket: {}", e);
                return;
            }
        };

        let code = self.generate_email_pin();

        if let Err(e) = send_email(
            &accept.email_address,
            &accept.account_name,
            &get_lang_string!(&EMAILS.validation.subject, name = accept.account_name),
            &get_lang_string!(
                &EMAILS.validation.body,
                name = accept.account_name,
                code = code
            ),
        )
        .await
        {
            self.close(format!("Failed to send email: {}", e)).await;
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Accept,
                PacketFamily::Account,
                AccountAcceptServerPacket {
                    reply_code: crate::deep::AccountValidationReply::OK,
                },
            )
            .await;
    }

    pub async fn handle_account(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Accept => self.account_accept(reader).await,
            PacketAction::Create => self.account_create(reader).await,
            PacketAction::Request => self.account_request(reader).await,
            PacketAction::Agree => self.account_agree(reader).await,
            _ => error!("Unhandled packet Account_{:?}", action),
        }
    }
}
