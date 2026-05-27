use crate::{
    SETTINGS,
    db::{DbHandle, insert_params},
    utils::{get_board_tile_spec, truncate_to_chars},
};
use chrono::{Duration, Utc};

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

        let subject = if subject.chars().count() > SETTINGS.load().board.max_subject_length as usize
        {
            truncate_to_chars(&subject, SETTINGS.load().board.max_subject_length as usize)
        } else {
            subject
        };

        let body = if body.chars().count() > SETTINGS.load().board.max_post_length as usize {
            truncate_to_chars(&body, SETTINGS.load().board.max_post_length as usize)
        } else {
            body
        };

        let character_id = character.id;

        let db = self.db.clone();
        tokio::spawn(async move {
            let (recent_posts, total_posts) =
                match get_board_post_counts(&db, board_id, character_id).await {
                    Ok((recent_posts, total_posts)) => (recent_posts, total_posts),
                    Err(e) => {
                        tracing::error!("Failed to get board post counts: {}", e);
                        return;
                    }
                };

            let map = match player.get_map().await {
                Ok(map) => map,
                Err(e) => {
                    tracing::error!("Failed to get map: {}", e);
                    return;
                }
            };

            if recent_posts >= SETTINGS.load().board.max_recent_posts
                || total_posts >= SETTINGS.load().board.max_user_posts
            {
                return map.open_board(player_id, board_id);
            }

            if let Err(e) = insert_post(&db, board_id, character_id, subject, body).await {
                tracing::error!("Failed to insert post: {}", e);
            }

            map.open_board(player_id, board_id);
        });
    }
}

async fn get_board_post_counts(
    db: &DbHandle,
    board_id: i32,
    character_id: i32,
) -> anyhow::Result<(i32, i32)> {
    let limit = if board_id == SETTINGS.load().board.admin_board {
        SETTINGS.load().board.admin_max_posts
    } else {
        SETTINGS.load().board.max_posts
    };

    let cutoff_time =
        Utc::now().naive_utc() - Duration::minutes(SETTINGS.load().board.recent_post_time as i64);

    let recent_posts = db
        .query_int(&insert_params(
            include_str!("../../../sql/get_recent_post_count.sql"),
            &[
                ("board_id", &board_id),
                ("character_id", &character_id),
                ("cutoff_time", &cutoff_time),
            ],
        ))
        .await?
        .unwrap_or_default();

    let total_posts = db
        .query_int(&insert_params(
            include_str!("../../../sql/get_total_post_count.sql"),
            &[
                ("board_id", &board_id),
                ("character_id", &character_id),
                ("limit", &limit),
            ],
        ))
        .await?
        .unwrap_or_default();

    Ok((recent_posts, total_posts))
}

async fn insert_post(
    db: &DbHandle,
    board_id: i32,
    character_id: i32,
    subject: String,
    body: String,
) -> anyhow::Result<()> {
    db.execute(&insert_params(
        include_str!("../../../sql/create_board_post.sql"),
        &[
            ("board_id", &board_id),
            ("character_id", &character_id),
            ("subject", &subject),
            ("body", &body),
        ],
    ))
    .await
}
