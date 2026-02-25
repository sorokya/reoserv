use crate::db::{insert_params, DbHandle};

pub async fn update_last_login_ip(db: &DbHandle, account_id: i32, ip: &str) -> anyhow::Result<()> {
    db.execute(&insert_params(
        include_str!("../../../sql/update_last_login_ip.sql"),
        &[("account_id", &account_id), ("ip", &ip.to_string())],
    ))
    .await?;

    Ok(())
}
