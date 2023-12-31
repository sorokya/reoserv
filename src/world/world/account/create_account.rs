use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::client::AccountCreateClientPacket;
use eolib::protocol::net::server::{
    AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
    AccountReplyServerPacketReplyCodeDataCreated, AccountReplyServerPacketReplyCodeDataExists,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use mysql_async::prelude::*;

use crate::errors::WrongSessionIdError;

use super::super::World;

use super::{account_exists::account_exists, password_hash::generate_password_hash};

impl World {
    pub async fn create_account(&self, player_id: i32, details: AccountCreateClientPacket) {
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

            let session_id = match player.take_session_id().await {
                Ok(session_id) => session_id,
                Err(e) => {
                    player.close(format!("Error getting session id: {}", e));
                    return;
                }
            };

            if session_id != details.session_id {
                player.close(format!(
                    "{}",
                    WrongSessionIdError::new(session_id, details.session_id)
                ));
                return;
            }

            // TODO: validate name

            let exists = match account_exists(&mut conn, &details.username).await {
                Ok(exists) => exists,
                Err(e) => {
                    player.close(format!("Error checking if account exists: {}", e));
                    return;
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
                reply.serialize(&mut writer);
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    writer.to_byte_array(),
                );
                return;
            }

            let password_hash = generate_password_hash(&details.username, &details.password);
            let player_ip = player.get_ip_addr().await;
            if let Err(e) = player_ip {
                player.close(format!("Error getting player ip: {}", e));
                return;
            }

            let player_ip = player_ip.unwrap();

            match conn
                .exec_drop(
                    include_str!("../../../sql/create_account.sql"),
                    params! {
                        "name" => &details.username,
                        "password_hash" => &password_hash,
                        "real_name" => &details.full_name,
                        "location" => &details.location,
                        "email" => &details.email,
                        "computer" => &details.computer,
                        "hdid" => &details.hdid,
                        "register_ip" => &player_ip,
                    },
                )
                .await
            {
                Ok(_) => {
                    info!("New account: {}", details.username);
                    let reply = AccountReplyServerPacket {
                        reply_code: AccountReply::Created,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Created(
                            AccountReplyServerPacketReplyCodeDataCreated::new(),
                        )),
                    };

                    let mut writer = EoWriter::new();
                    reply.serialize(&mut writer);
                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        writer.to_byte_array(),
                    );
                }
                Err(e) => {
                    player.close(format!("Error creating account: {}", e));
                }
            }
        });
    }
}
