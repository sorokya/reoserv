use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        PacketAction, PacketFamily,
        server::{
            AdminInteractReplyServerPacket, AdminInteractReplyServerPacketMessageTypeData,
            AdminInteractReplyServerPacketMessageTypeDataMessage, AdminMessageType,
        },
    },
};

use crate::{SETTINGS, db::insert_params, utils::capitalize};

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
                tracing::error!("Failed to get character: {}", e);
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
            tracing::error!("Failed to serialize AdminInteractReplyServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for player in self.players.values() {
            if let Ok(character) = player.get_character().await
                && character.name != player_name
                && i32::from(character.admin_level) >= 1
            {
                player.send_buf(
                    PacketAction::Reply,
                    PacketFamily::AdminInteract,
                    buf.clone(),
                );
            }
        }
    }

    fn add_message_to_admin_board(&self, character_id: i32, player_name: String, message: String) {
        let db = self.db.clone();
        tokio::spawn(async move {
            if let Err(e) = db
                .execute(&insert_params(
                    include_str!("../../../sql/create_board_post.sql"),
                    &[
                        ("board_id", &SETTINGS.board.admin_board),
                        ("character_id", &character_id),
                        (
                            "subject",
                            &format!("[Request] {} needs help", capitalize(&player_name)),
                        ),
                        ("body", &message),
                    ],
                ))
                .await
            {
                tracing::error!("Failed to add message to admin board: {}", e);
            }
        });
    }
}
