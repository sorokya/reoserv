use mysql_async::{prelude::*, Conn};
use rand::{rngs::OsRng, RngCore};

pub async fn generate_session(
    conn: &mut Conn,
    account_id: i32,
) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    let token = hex::encode(bytes);

    conn.exec_drop(
        include_str!("../../../sql/generate_session.sql"),
        params! {
            "account_id" => account_id,
            "token" => &token,
        },
    )
    .await?;

    Ok(token)
}
