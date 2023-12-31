use eolib::{data::{EoWriter, EoSerialize}, protocol::net::{PacketAction, PacketFamily, server::BoardPlayerServerPacket}};
use mysql_async::{params, prelude::Queryable, Row};

use crate::utils::get_board_tile_spec;

use super::super::Map;

impl Map {
    pub async fn view_board_post(&self, player_id: i32, post_id: i32) {
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
            let mut writer = EoWriter::new();

            let mut conn = pool.get_conn().await.unwrap();

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

            let packet = BoardPlayerServerPacket {
                post_id,
                post_body: row.take::<String, &str>("body").unwrap(),
            };

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize BoardPlayerServerPacket: {}", e);
                return;
            }

            player.send(PacketAction::Player, PacketFamily::Board, writer.to_byte_array());
        });
    }
}
