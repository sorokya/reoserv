use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::client::AccountCreateClientPacket;
use eolib::protocol::net::server::{
    AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
    AccountReplyServerPacketReplyCodeDataCreated, AccountReplyServerPacketReplyCodeDataExists,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use mysql_async::prelude::*;

use crate::errors::WrongSessionIdError;
use crate::player::ClientState;

use super::super::Player;

use super::{account_exists::account_exists, password_hash::generate_password_hash};

impl Player {
    pub async fn create_account(&mut self, packet: AccountCreateClientPacket) -> bool {
        if self.state != ClientState::Accepted {
            return true;
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

        let session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return false;
            }
        };

        if session_id != packet.session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(session_id, packet.session_id)
            ))
            .await;
            return false;
        }

        // TODO: validate name

        let exists = match account_exists(&mut conn, &packet.username).await {
            Ok(exists) => exists,
            Err(e) => {
                self.close(format!("Error checking if account exists: {}", e))
                    .await;
                return false;
            }
        };

        if exists {
            let reply = AccountReplyServerPacket {
                reply_code: AccountReply::Exists,
                reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                    AccountReplyServerPacketReplyCodeDataExists::new(),
                )),
            };

            let mut writer = EoWriter::new();

            if let Err(e) = reply.serialize(&mut writer) {
                self.close(format!(
                    "Failed to serialize AccountReplyServerPacket: {}",
                    e
                ))
                .await;
                return false;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    writer.to_byte_array(),
                )
                .await;
            return true;
        }

        let password_hash = generate_password_hash(&packet.username, &packet.password);

        match conn
            .exec_drop(
                include_str!("../../../sql/create_account.sql"),
                params! {
                    "name" => &packet.username,
                    "password_hash" => &password_hash,
                    "real_name" => &packet.full_name,
                    "location" => &packet.location,
                    "email" => &packet.email,
                    "computer" => &packet.computer,
                    "hdid" => &packet.hdid,
                    "register_ip" => &self.ip,
                },
            )
            .await
        {
            Ok(_) => {
                info!("New account: {}", packet.username);
                let reply = AccountReplyServerPacket {
                    reply_code: AccountReply::Created,
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Created(
                        AccountReplyServerPacketReplyCodeDataCreated::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = reply.serialize(&mut writer) {
                    self.close(format!(
                        "Failed to serialize AccountReplyServerPacket: {}",
                        e
                    ))
                    .await;
                    return false;
                }

                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        writer.to_byte_array(),
                    )
                    .await;
                true
            }
            Err(e) => {
                self.close(format!("Error creating account: {}", e)).await;
                false
            }
        }
    }
}
