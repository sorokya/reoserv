use chrono::NaiveDateTime;
use eolib::protocol::net::{
    server::{BoardOpenServerPacket, BoardPostListing},
    PacketAction, PacketFamily,
};
use mysql_async::{params, prelude::Queryable, Row};

use crate::{
    utils::{format_duration, get_board_tile_spec},
    SETTINGS,
};

use super::super::Map;

struct BoardPost {
    id: i32,
    author: String,
    subject: String,
    created_at: NaiveDateTime,
}

impl Map {
    pub fn open_board(&mut self, player_id: i32, board_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let board_tile_spec = match get_board_tile_spec(board_id) {
            Some(spec) => spec,
            None => return,
        };

        if !self.player_in_range_of_tile(player_id, board_tile_spec) {
            return;
        }

        if board_id == SETTINGS.board.admin_board && i32::from(character.admin_level) < 1 {
            return;
        }

        let player = match &character.player {
            Some(player) => player.clone(),
            None => return,
        };

        player.set_board_id(board_id);

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get connection from pool: {}", e);
                    return;
                }
            };

            let limit = if board_id == SETTINGS.board.admin_board {
                SETTINGS.board.admin_max_posts
            } else {
                SETTINGS.board.max_posts
            };

            let posts = match conn
                .exec_map(
                    include_str!("../../../sql/get_board_posts.sql"),
                    params! {
                        "board_id" => board_id,
                        "limit" => limit,
                    },
                    |mut row: Row| BoardPost {
                        id: row.take("id").unwrap(),
                        author: row.take("author").unwrap(),
                        subject: row.take("subject").unwrap(),
                        created_at: row.take("created_at").unwrap(),
                    },
                )
                .await
            {
                Ok(posts) => posts,
                Err(e) => {
                    error!("Failed to get board posts: {}", e);
                    return;
                }
            };

            player.send(
                PacketAction::Open,
                PacketFamily::Board,
                &BoardOpenServerPacket {
                    board_id,
                    posts: posts
                        .iter()
                        .map(|post| BoardPostListing {
                            post_id: post.id,
                            author: post.author.to_owned(),
                            subject: if SETTINGS.board.date_posts {
                                format!("{} ({})", post.subject, format_duration(&post.created_at))
                            } else {
                                post.subject.to_owned()
                            },
                        })
                        .collect(),
                },
            );
        });
    }
}
