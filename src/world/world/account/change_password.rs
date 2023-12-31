use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
            AccountReplyServerPacketReplyCodeDataChangeFailed,
            AccountReplyServerPacketReplyCodeDataChanged,
            AccountReplyServerPacketReplyCodeDataExists,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::{prelude::*, Params, Row};

use super::{
    super::World,
    password_hash::{generate_password_hash, validate_password},
};

use super::account_exists::account_exists;

impl World {
    pub fn change_password(
        &self,
        player_id: i32,
        username: String,
        current_password: String,
        new_password: String,
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
                    player.close(format!("Error getting connection from pool: {}", e));
                    return;
                }
            };

            let exists = match account_exists(&mut conn, &username).await {
                Ok(exists) => exists,
                Err(e) => {
                    player.close(format!("Error checking if account exists: {}", e));
                    return;
                }
            };

            if !exists {
                let packet = AccountReplyServerPacket {
                    reply_code: AccountReply::Exists,
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                        AccountReplyServerPacketReplyCodeDataExists::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize AccountReplyServerPacket: {}", e);
                    return;
                }

                player.send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    writer.to_byte_array(),
                );
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

                    let packet = AccountReplyServerPacket {
                        reply_code: AccountReply::ChangeFailed,
                        reply_code_data: Some(AccountReplyServerPacketReplyCodeData::ChangeFailed(
                            AccountReplyServerPacketReplyCodeDataChangeFailed::new(),
                        )),
                    };

                    let mut writer = EoWriter::new();

                    if let Err(e) = packet.serialize(&mut writer) {
                        error!("Failed to serialize AccountReplyServerPacket: {}", e);
                        return;
                    }

                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Account,
                        writer.to_byte_array(),
                    );
                    return;
                }
            }
            .unwrap();

            let password_hash: String = row.get("password_hash").unwrap();
            if !validate_password(&username, &current_password, &password_hash) {
                let packet = AccountReplyServerPacket {
                    reply_code: AccountReply::ChangeFailed,
                    reply_code_data: Some(AccountReplyServerPacketReplyCodeData::ChangeFailed(
                        AccountReplyServerPacketReplyCodeDataChangeFailed::new(),
                    )),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize AccountReplyServerPacket: {}", e);
                    return;
                }

                player.send(
                    PacketAction::Reply,
                    PacketFamily::Account,
                    writer.to_byte_array(),
                );
                return;
            }

            let account_id: i32 = row.get("id").unwrap();

            let password_hash = generate_password_hash(&username, &new_password);
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
                player.close(format!("Error updating password hash: {}", e));
                return;
            }

            let packet = AccountReplyServerPacket {
                reply_code: AccountReply::Changed,
                reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Changed(
                    AccountReplyServerPacketReplyCodeDataChanged::new(),
                )),
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize AccountReplyServerPacket: {}", e);
                return;
            }

            player.send(
                PacketAction::Reply,
                PacketFamily::Account,
                writer.to_byte_array(),
            );
        });
    }
}
