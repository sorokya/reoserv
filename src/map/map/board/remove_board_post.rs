use eo::data::EOShort;
use mysql_async::{params, prelude::Queryable, Conn};

use crate::utils::get_board_tile_spec;

use super::super::Map;

impl Map {
    pub async fn remove_board_post(&mut self, player_id: EOShort, post_id: EOShort) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let board_id = match character.player.as_ref().unwrap().get_board_id().await {
            Some(board_id) => board_id,
            None => return,
        };

        if character.admin_level.to_char() < 1 {
            return self.open_board(player_id, board_id);
        }

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
            let mut conn = pool.get_conn().await.unwrap();

            let map = match player.get_map().await {
                Ok(map) => map,
                Err(e) => {
                    error!("Failed to get map: {}", e);
                    return;
                }
            };

            if let Err(e) = delete_post(&mut conn, post_id).await {
                error!("Failed to delete post: {}", e);
            }

            map.open_board(player_id, board_id);
        });
    }
}

async fn delete_post(conn: &mut Conn, post_id: EOShort) -> Result<(), mysql_async::Error> {
    conn.exec_drop(
        include_str!("../../../sql/delete_board_post.sql"),
        params! {
            "id" => post_id,
        },
    )
    .await
}
