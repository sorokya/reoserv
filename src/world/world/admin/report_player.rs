use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            AdminInteractReplyServerPacket, AdminInteractReplyServerPacketMessageTypeData,
            AdminInteractReplyServerPacketMessageTypeDataReport, AdminMessageType,
        },
        PacketAction, PacketFamily,
    },
};
use mysql_async::prelude::Queryable;
use mysql_common::params;

use super::super::World;
use crate::{utils::capitalize, SETTINGS};

impl World {
    pub async fn report_player(&self, player_id: i32, reportee_name: String, message: String) {
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

        self.notify_report_to_online_admins(&character.name, &message, &reportee_name)
            .await;

        self.add_report_to_admin_board(
            character.id,
            character.name.clone(),
            message,
            reportee_name,
        );
    }

    async fn notify_report_to_online_admins(
        &self,
        player_name: &str,
        message: &str,
        reportee_name: &str,
    ) {
        let packet = AdminInteractReplyServerPacket {
            message_type: AdminMessageType::Report,
            message_type_data: Some(AdminInteractReplyServerPacketMessageTypeData::Report(
                AdminInteractReplyServerPacketMessageTypeDataReport {
                    player_name: player_name.to_owned(),
                    message: message.to_owned(),
                    reportee_name: reportee_name.to_owned(),
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
                    player.send_buf(
                        PacketAction::Reply,
                        PacketFamily::AdminInteract,
                        buf.clone(),
                    );
                }
            }
        }
    }

    fn add_report_to_admin_board(
        &self,
        character_id: i32,
        player_name: String,
        message: String,
        reportee_name: String,
    ) {
        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.get_conn().await.unwrap();
            conn.exec_drop(
                include_str!("../../../sql/create_board_post.sql"),
                params! {
                    "board_id" => SETTINGS.board.admin_board,
                    "character_id" => character_id,
                    "subject" => format!("[Report] {} reports {}", capitalize(&player_name), capitalize(&reportee_name)),
                    "body" => message,
                },
            )
            .await
        });
    }
}
