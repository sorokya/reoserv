use eolib::protocol::net::{server::BoardPlayerServerPacket, PacketAction, PacketFamily};
use mysql_async::{params, prelude::Queryable, Row};

use crate::utils::get_board_tile_spec;

use super::super::Map;

impl Map {
    pub fn view_board_post(&self, player_id: i32, board_id: i32, post_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player.clone(),
            None => return,
        };

        let board_tile_spec = match get_board_tile_spec(board_id) {
            Some(spec) => spec,
            None => return,
        };

        if !self.player_in_range_of_tile(player_id, board_tile_spec) {
            return;
        }

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get sql connection: {}", e);
                    return;
                }
            };

            let row = conn
                .exec_first(
                    include_str!("../../../sql/get_board_post.sql"),
                    params! {
                        "board_id" => board_id,
                        "post_id" => post_id,
                    },
                )
                .await;

            let mut row: Row = match row {
                Ok(Some(row)) => row,
                _ => return,
            };

            player.send(
                PacketAction::Player,
                PacketFamily::Board,
                &BoardPlayerServerPacket {
                    post_id,
                    post_body: row.take::<String, &str>("body").unwrap(),
                },
            );
        });
    }
}
