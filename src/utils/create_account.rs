use mysql_async::{prelude::*, Conn};

pub struct CreateAccountParams {
    pub name: String,
    pub password_hash: String,
    pub real_name: String,
    pub location: String,
    pub email: String,
    pub computer: String,
    pub hdid: String,
    pub register_ip: String,
}

pub async fn create_account(
    conn: &mut Conn,
    params: CreateAccountParams,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.exec_drop(
        include_str!("../sql/create_account.sql"),
        params! {
            "name" => &params.name,
            "password_hash" => &params.password_hash,
            "real_name" => &params.real_name,
            "location" => &params.location,
            "email" => &params.email,
            "computer" => &params.computer,
            "hdid" => &params.hdid,
            "register_ip" => &params.register_ip,
        },
    )
    .await?;
    Ok(())
}
