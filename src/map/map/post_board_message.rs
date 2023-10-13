use eo::data::{EOInt, EOShort};
use mysql_async::{params, prelude::Queryable, Conn, Row};

use crate::{errors::DataNotFoundError, utils::get_board_tile_spec, SETTINGS};

use super::Map;

impl Map {
    pub async fn post_board_message(&mut self, player_id: EOShort, subject: String, body: String) {
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
            None => return self.open_board(player_id, board_id),
        };

        if !self.player_in_range_of_tile(player_id, board_tile_spec) {
            return self.open_board(player_id, board_id);
        }

        let subject = if subject.len() > SETTINGS.board.max_subject_length as usize {
            String::from(&subject[..SETTINGS.board.max_subject_length as usize])
        } else {
            subject
        };

        let body = if body.len() > SETTINGS.board.max_post_length as usize {
            String::from(&body[..SETTINGS.board.max_post_length as usize])
        } else {
            body
        };

        let character_id = character.id;

        let player = match &character.player {
            Some(player) => player.clone(),
            None => return,
        };

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.get_conn().await.unwrap();
            let (recent_posts, total_posts) =
                match get_board_post_counts(&mut conn, board_id, character_id).await {
                    Ok((recent_posts, total_posts)) => (recent_posts, total_posts),
                    Err(e) => {
                        error!("Failed to get board post counts: {}", e);
                        return;
                    }
                };

            let map = match player.get_map().await {
                Ok(map) => map,
                Err(e) => {
                    error!("Failed to get map: {}", e);
                    return;
                }
            };

            if recent_posts >= SETTINGS.board.max_recent_posts
                || total_posts >= SETTINGS.board.max_user_posts
            {
                return map.open_board(player_id, board_id);
            }

            match insert_post(&mut conn, board_id, character_id, subject, body).await {
                Ok(_) => {}
                Err(e) => {
                    error!("Failed to insert post: {}", e);
                }
            }

            map.open_board(player_id, board_id);
        });
    }
}

async fn get_board_post_counts(
    conn: &mut Conn,
    board_id: EOShort,
    character_id: EOInt,
) -> Result<(EOInt, EOInt), Box<dyn std::error::Error>> {
    let limit = if board_id == SETTINGS.board.admin_board as EOShort {
        SETTINGS.board.admin_max_posts
    } else {
        SETTINGS.board.max_posts
    };

    let mut row: Row = match conn
        .exec_first(
            include_str!("../../sql/get_recent_post_count.sql"),
            params! {
                "board_id" => board_id,
                "character_id" => character_id,
                "post_time" => SETTINGS.board.recent_post_time,
            },
        )
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err(Box::new(DataNotFoundError {
                kind: "BoardPost".to_string(),
                id: character_id as EOShort,
            }))
        }
        Err(e) => {
            error!("Failed to get recent post count: {}", e);
            return Err(Box::new(e));
        }
    };

    let recent_posts: EOInt = row.take("recent_posts").unwrap_or(0);

    let mut row: Row = match conn
        .exec_first(
            include_str!("../../sql/get_total_post_count.sql"),
            params! {
                "board_id" => board_id,
                "character_id" => character_id,
                "limit" => limit,
            },
        )
        .await
    {
        Ok(Some(row)) => row,
        Ok(None) => {
            return Err(Box::new(DataNotFoundError {
                kind: "BoardPost".to_string(),
                id: character_id as EOShort,
            }))
        }
        Err(e) => {
            error!("Failed to get recent post count: {}", e);
            return Err(Box::new(e));
        }
    };

    let total_posts: EOInt = row.take("total_posts").unwrap_or(0);

    Ok((recent_posts, total_posts))
}

async fn insert_post(
    conn: &mut Conn,
    board_id: EOShort,
    character_id: EOInt,
    subject: String,
    body: String,
) -> Result<(), Box<dyn std::error::Error>> {
    match conn
        .exec_drop(
            include_str!("../../sql/create_board_post.sql"),
            params! {
                "board_id" => board_id,
                "character_id" => character_id,
                "subject" => subject,
                "body" => body,
            },
        )
        .await
    {
        Ok(_) => Ok(()),
        Err(e) => Err(Box::new(e)),
    }
}
