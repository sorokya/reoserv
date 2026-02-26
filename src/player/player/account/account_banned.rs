use std::time::Duration;

use chrono::Utc;

use crate::db::{DbHandle, insert_params};

pub async fn account_banned(db: &DbHandle, name: &str) -> anyhow::Result<bool> {
    let row = match db
        .query_one(&insert_params(
            include_str!("../../../sql/get_account_ban_duration.sql"),
            &[("name", &name.to_string())],
        ))
        .await?
    {
        Some(row) => row,
        _ => {
            return Ok(false);
        }
    };

    let duration = row.get_int(0).unwrap();
    // 0 = permanent
    if duration == 0 {
        return Ok(true);
    }

    let now = Utc::now().naive_utc();
    let created_at = row.get_date(1).unwrap();
    let expires_at = created_at + Duration::from_mins(duration as u64);
    let diff = expires_at - now;
    let remaining = diff.num_minutes();
    if remaining <= 0 {
        return Ok(false);
    }

    Ok(true)
}
