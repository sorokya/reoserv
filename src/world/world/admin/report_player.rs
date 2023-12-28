use eolib::{data::EoWriter, protocol::net::{server::AdminMessageType, PacketAction, PacketFamily}};
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

        let mut writer = EoWriter::new();
        writer.add_char(i32::from(AdminMessageType::Report));
        writer.add_byte(0xff);
        writer.add_string(&character.name);
        writer.add_byte(0xff);
        writer.add_string(&message);
        writer.add_byte(0xff);
        writer.add_string(&reportee_name);
        writer.add_byte(0xff);

        let from_name = character.name;
        let buf = writer.to_byte_array();

        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != from_name && i32::from(character.admin_level) >= 1 {
                    player.send(
                        PacketAction::Reply,
                        PacketFamily::AdminInteract,
                        buf.clone(),
                    );
                }
            }
        }

        let pool = self.pool.clone();
        let character_id = character.id;
        tokio::spawn(async move {
            let mut conn = pool.get_conn().await.unwrap();
            conn.exec_drop(
                include_str!("../../../sql/create_board_post.sql"),
                params! {
                    "board_id" => SETTINGS.board.admin_board,
                    "character_id" => character_id,
                    "subject" => format!("[Report] {} reports {}", capitalize(&from_name), capitalize(&reportee_name)),
                    "body" => message,
                },
            )
            .await
        });
    }
}
