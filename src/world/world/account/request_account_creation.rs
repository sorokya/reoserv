use eo::{
    data::{i32, i32, Serializeable, StreamBuilder},
    protocol::{
        server::account::{self, Reply},
        AccountReply, PacketAction, PacketFamily,
    },
};

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
            let reply = Reply {
                reply_code: AccountReply::Exists,
                data: account::ReplyData::Exists(account::ReplyExists {
                    no: "NO".to_string(),
                }),
            };
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
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

        let reply = Reply {
            reply_code: AccountReply::SessionId(session_id),
            data: account::ReplyData::SessionId(account::ReplySessionId {
                ok: "OK".to_string(),
                sequence_start: sequence_start as i32,
            }),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
    }
}
