use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use eo::data::EOInt;
use mysql_async::prelude::*;
use mysql_common::{params, Row};

use super::Player;

impl Player {
    pub async fn get_ban_duration(&mut self) -> Option<EOInt> {
        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(err) => {
                error!("Failed to get connection from pool: {}", err);
                return None;
            }
        };

        let mut row: Row = match conn
            .exec_first(
                include_str!("../../sql/get_ban_duration.sql"),
                params! {
                    "ip" => &self.ip,
                },
            )
            .await
        {
            Ok(Some(row)) => row,
            _ => return None,
        };

        let duration: EOInt = row.take("duration").unwrap();
        // 0 = permanent
        if duration == 0 {
            return Some(0);
        }

        let now = Utc::now();
        let created_at: NaiveDateTime = row.take("created_at").unwrap();
        let expires_at = created_at + Duration::from_secs(duration as u64 * 60);
        let diff = expires_at - now.naive_utc();
        let remaining = diff.num_minutes();
        if remaining <= 0 {
            return None;
        }

        Some(diff.num_minutes() as EOInt)
    }
}
