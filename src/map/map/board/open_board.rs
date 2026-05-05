use chrono::NaiveDateTime;
use eolib::protocol::net::{
    PacketAction, PacketFamily,
    server::{BoardOpenServerPacket, BoardPostListing},
};

use crate::{
    SETTINGS,
    db::insert_params,
    utils::{format_duration, get_board_tile_spec},
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

        if board_id == SETTINGS.load().board.admin_board && i32::from(character.admin_level) < 1 {
            return;
        }

        let player = match &character.player {
            Some(player) => player.clone(),
            None => return,
        };

        player.set_board_id(board_id);

        let db = self.db.clone();
        tokio::spawn(async move {
            let limit = if board_id == SETTINGS.load().board.admin_board {
                SETTINGS.load().board.admin_max_posts
            } else {
                SETTINGS.load().board.max_posts
            };

            let posts = match db
                .query_map(
                    &insert_params(
                        include_str!("../../../sql/get_board_posts.sql"),
                        &[("board_id", &board_id), ("limit", &limit)],
                    ),
                    |row| BoardPost {
                        id: row.get_int(0).unwrap(),
                        author: row.get_string(1).unwrap(),
                        subject: row.get_string(2).unwrap(),
                        created_at: row.get_date(3).unwrap(),
                    },
                )
                .await
            {
                Ok(posts) => posts,
                Err(e) => {
                    tracing::error!("Failed to get board posts: {}", e);
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
                            subject: if SETTINGS.load().board.date_posts {
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
