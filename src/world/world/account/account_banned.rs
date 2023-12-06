use std::time::Duration;

use chrono::{NaiveDateTime, Utc};
use eo::data::EOInt;
use mysql_async::{params, prelude::Queryable, Conn, Params, Row};

pub async fn account_banned(
    conn: &mut Conn,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    let mut row: Row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("../../../sql/get_account_ban_duration.sql"),
            params! {
                "name" => name,
            },
        )
        .await?
    {
        Some(row) => row,
        _ => {
            return Ok(false);
        }
    };

    let duration: EOInt = row.take("duration").unwrap();
    // 0 = permanent
    if duration == 0 {
        return Ok(true);
    }

    let now = Utc::now();
    let created_at: NaiveDateTime = row.take("created_at").unwrap();
    let expires_at = created_at + Duration::from_secs(duration as u64 * 60);
    let diff = expires_at - now.naive_utc();
    let remaining = diff.num_minutes();
    if remaining <= 0 {
        return Ok(false);
    }

    Ok(true)
}
