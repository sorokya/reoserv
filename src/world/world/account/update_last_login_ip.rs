use eo::data::EOInt;
use mysql_async::{prelude::*, Conn};

pub async fn update_last_login_ip(
    conn: &mut Conn,
    account_id: EOInt,
    ip: &str,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    conn.exec_drop(
        include_str!("../../../sql/update_last_login_ip.sql"),
        params! {
            "account_id" => &account_id,
            "ip" => &ip,
        },
    )
    .await?;

    Ok(())
}
