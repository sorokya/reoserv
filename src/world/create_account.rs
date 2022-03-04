use mysql_async::{prelude::*, Conn};

pub async fn create_account(
    conn: &mut Conn,
    name: String,
    password_hash: String,
    real_name: String,
    location: String,
    email: String,
    computer: String,
    hdid: String,
    register_ip: String,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    conn.exec_drop(
        include_str!("../sql/create_account.sql"),
        params! {
            "name" => &name,
            "password_hash" => &password_hash,
            "real_name" => &real_name,
            "location" => &location,
            "email" => &email,
            "computer" => &computer,
            "hdid" => &hdid,
            "register_ip" => &register_ip,
        },
    )
    .await?;
    Ok(())
}
