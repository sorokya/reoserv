use eolib::protocol::net::{PacketAction, PacketFamily, server::BoardPlayerServerPacket};

use crate::{db::insert_params, utils::get_board_tile_spec};

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

        let db = self.db.clone();
        tokio::spawn(async move {
            let post_body = match db
                .query_string(&insert_params(
                    include_str!("../../../sql/get_board_post.sql"),
                    &[("board_id", &board_id), ("post_id", &post_id)],
                ))
                .await
            {
                Ok(Some(body)) => body,
                Ok(None) => "".to_string(),
                Err(e) => {
                    tracing::error!("Failed to query board post: {}", e);
                    return;
                }
            };

            player.send(
                PacketAction::Player,
                PacketFamily::Board,
                &BoardPlayerServerPacket { post_id, post_body },
            );
        });
    }
}
