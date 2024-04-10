use mysql_async::{params, prelude::Queryable, Conn, Row};

use crate::{errors::DataNotFoundError, utils::get_board_tile_spec, SETTINGS};

use super::super::Map;

impl Map {
    pub fn create_board_post(
        &mut self,
        player_id: i32,
        board_id: i32,
        subject: String,
        body: String,
    ) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match &character.player {
            Some(player) => player.clone(),
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

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Failed to get sql connection: {}", e);
                    return;
                }
            };

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

            if let Err(e) = insert_post(&mut conn, board_id, character_id, subject, body).await {
                error!("Failed to insert post: {}", e);
            }

            map.open_board(player_id, board_id);
        });
    }
}

async fn get_board_post_counts(
    conn: &mut Conn,
    board_id: i32,
    character_id: i32,
) -> Result<(i32, i32), Box<dyn std::error::Error>> {
    let limit = if board_id == SETTINGS.board.admin_board {
        SETTINGS.board.admin_max_posts
    } else {
        SETTINGS.board.max_posts
    };

    let mut row: Row = match conn
        .exec_first(
            include_str!("../../../sql/get_recent_post_count.sql"),
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
                id: character_id,
            }))
        }
        Err(e) => {
            error!("Failed to get recent post count: {}", e);
            return Err(Box::new(e));
        }
    };

    let recent_posts: i32 = row.take("recent_posts").unwrap_or(0);

    let mut row: Row = match conn
        .exec_first(
            include_str!("../../../sql/get_total_post_count.sql"),
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
                id: character_id,
            }))
        }
        Err(e) => {
            error!("Failed to get recent post count: {}", e);
            return Err(Box::new(e));
        }
    };

    let total_posts: i32 = row.take("total_posts").unwrap_or(0);

    Ok((recent_posts, total_posts))
}

async fn insert_post(
    conn: &mut Conn,
    board_id: i32,
    character_id: i32,
    subject: String,
    body: String,
) -> Result<(), mysql_async::Error> {
    conn.exec_drop(
        include_str!("../../../sql/create_board_post.sql"),
        params! {
            "board_id" => board_id,
            "character_id" => character_id,
            "subject" => subject,
            "body" => body,
        },
    )
    .await
}
