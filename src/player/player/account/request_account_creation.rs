
use eolib::protocol::net::server::{
    AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
    AccountReplyServerPacketReplyCodeDataDefault, AccountReplyServerPacketReplyCodeDataExists,
};
use eolib::protocol::net::{PacketAction, PacketFamily};

use crate::player::ClientState;

use super::account_exists::account_exists;

use super::super::Player;

impl Player {
    pub async fn request_account_creation(&mut self, username: String) -> bool {
        if self.state != ClientState::Accepted {
            return true;
        }

        // TODO: validate name

        let mut conn = match self.pool.get_conn().await {
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
            return true;
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

        true
    }
}
