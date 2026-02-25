use std::time::Duration;

use chrono::{NaiveDateTime, Utc};

use crate::db::insert_params;

use super::Player;

impl Player {
    pub async fn get_ban_duration(&mut self) -> Option<i32> {
        let row = match self
            .db
            .query_one(&insert_params(
                include_str!("../../sql/get_ban_duration.sql"),
                &[("ip", &self.ip)],
            ))
            .await
        {
            Ok(Some(row)) => row,
            _ => return None,
        };

        let duration = match row.get_int(0) {
            Some(d) => d,
            None => return None,
        };

        // 0 = permanent
        if duration == 0 {
            return Some(0);
        }

        let now = Utc::now().naive_utc();
        let created_at: NaiveDateTime = row.get_date(1).unwrap();
        let expires_at = created_at + Duration::from_mins(duration as u64);
        let diff = expires_at - now;
        let remaining = diff.num_minutes();
        if remaining <= 0 {
            return None;
        }

        Some(diff.num_minutes() as i32)
    }
}
