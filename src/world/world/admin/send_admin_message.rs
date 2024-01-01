use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            AdminInteractReplyServerPacket, AdminInteractReplyServerPacketMessageTypeData,
            AdminInteractReplyServerPacketMessageTypeDataMessage, AdminMessageType,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::prelude::Queryable;
use mysql_common::params;

use crate::{utils::capitalize, SETTINGS};

use super::super::World;

impl World {
    pub async fn send_admin_message(&self, player_id: i32, message: String) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let character = match player.get_character().await {
            Ok(character) => character,
            Err(e) => {
                error!("Failed to get character: {}", e);
                return;
            }
        };

        self.notify_message_to_online_admins(&character.name, &message)
            .await;

        self.add_message_to_admin_board(character.id, character.name.clone(), message);
    }

    async fn notify_message_to_online_admins(&self, player_name: &str, message: &str) {
        let packet = AdminInteractReplyServerPacket {
            message_type: AdminMessageType::Message,
            message_type_data: Some(AdminInteractReplyServerPacketMessageTypeData::Message(
                AdminInteractReplyServerPacketMessageTypeDataMessage {
                    player_name: player_name.to_owned(),
                    message: message.to_owned(),
                },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize AdminInteractReplyServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != player_name && i32::from(character.admin_level) >= 1 {
                    player.send(
                        PacketAction::Reply,
                        PacketFamily::AdminInteract,
                        buf.clone(),
                    );
                }
            }
        }
    }

    fn add_message_to_admin_board(&self, character_id: i32, player_name: String, message: String) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.get_conn().await.unwrap();
            conn.exec_drop(
                include_str!("../../../sql/create_board_post.sql"),
                params! {
                    "board_id" => SETTINGS.board.admin_board,
                    "character_id" => character_id,
                    "subject" => format!("[Request] {} needs help", capitalize(&player_name)),
                    "body" => message,
                },
            )
            .await
        });
    }
}
