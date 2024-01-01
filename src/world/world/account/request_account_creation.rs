use eolib::data::{EoSerialize, EoWriter};
use eolib::protocol::net::server::{
    AccountReply, AccountReplyServerPacket, AccountReplyServerPacketReplyCodeData,
    AccountReplyServerPacketReplyCodeDataDefault, AccountReplyServerPacketReplyCodeDataExists,
};
use eolib::protocol::net::{PacketAction, PacketFamily};

use super::account_exists::account_exists;

use super::super::World;

impl World {
    pub async fn request_account_creation(&self, player_id: i32, username: String) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        // TODO: validate name

        let mut conn = match self.pool.get_conn().await {
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

        if exists {
            let reply = AccountReplyServerPacket {
                reply_code: AccountReply::Exists,
                reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Exists(
                    AccountReplyServerPacketReplyCodeDataExists::new(),
                )),
            };
            let mut writer = EoWriter::new();

            if let Err(e) = reply.serialize(&mut writer) {
                player.close(format!("Error serializing reply: {}", e));
                return;
            }

            player.send(
                PacketAction::Reply,
                PacketFamily::Account,
                writer.to_byte_array(),
            );
            return;
        }

        let session_id = match player.generate_session_id().await {
            Ok(session_id) => session_id,
            Err(e) => {
                player.close(format!("Error generating session id: {}", e));
                return;
            }
        };

        let sequence_start = match player.get_sequence_start().await {
            Ok(sequence_start) => sequence_start,
            Err(e) => {
                player.close(format!("Error getting sequence start: {}", e));
                return;
            }
        };

        let reply = AccountReplyServerPacket {
            reply_code: AccountReply::Unrecognized(session_id),
            reply_code_data: Some(AccountReplyServerPacketReplyCodeData::Default(
                AccountReplyServerPacketReplyCodeDataDefault { sequence_start },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            player.close(format!("Error serializing reply: {}", e));
            return;
        }

        player.send(
            PacketAction::Reply,
            PacketFamily::Account,
            writer.to_byte_array(),
        );
    }
}
