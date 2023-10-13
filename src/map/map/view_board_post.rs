use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};
use mysql_async::{params, prelude::Queryable, Row};

use crate::utils::get_board_tile_spec;

use super::Map;

impl Map {
    pub async fn view_board_post(&self, player_id: EOShort, post_id: EOShort) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let board_id = match character.player.as_ref().unwrap().get_board_id().await {
            Some(board_id) => board_id,
            None => return,
        };

        let board_tile_spec = match get_board_tile_spec(board_id) {
            Some(spec) => spec,
            None => return,
        };

        if !self.player_in_range_of_tile(player_id, board_tile_spec) {
            return;
        }

        let player = match &character.player {
            Some(player) => player.clone(),
            None => return,
        };

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut builder = StreamBuilder::new();

            let mut conn = pool.get_conn().await.unwrap();

            let row = conn
                .exec_first(
                    include_str!("../../sql/get_board_post.sql"),
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

            builder.add_short(post_id);
            builder.add_string(&row.take::<String, &str>("body").unwrap());

            player.send(PacketAction::Player, PacketFamily::Board, builder.get());
        });
    }
}
