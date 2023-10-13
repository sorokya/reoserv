use eo::{
    data::{EOChar, EOShort, StreamBuilder, EO_BREAK_CHAR},
    protocol::{AdminMessageType, PacketAction, PacketFamily},
};
use mysql_async::prelude::Queryable;
use mysql_common::params;

use super::super::World;
use crate::{utils::capitalize, SETTINGS};

impl World {
    pub async fn report_player(&self, player_id: EOShort, reportee_name: String, message: String) {
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

        let mut builder = StreamBuilder::new();
        builder.add_char(AdminMessageType::Report.to_char());
        builder.add_byte(EO_BREAK_CHAR);
        builder.add_break_string(&character.name);
        builder.add_break_string(&message);
        builder.add_break_string(&reportee_name);

        let from_name = character.name;
        let buf = builder.get();

        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != from_name && character.admin_level as EOChar >= 1 {
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
