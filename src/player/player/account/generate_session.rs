use rand::Rng;

use crate::db::{DbHandle, insert_params};

pub async fn generate_session(db: &DbHandle, account_id: i32) -> anyhow::Result<String> {
    let token = {
        let mut bytes = [0u8; 32];
        let mut rng = rand::rng();
        rng.fill_bytes(&mut bytes);

        hex::encode(bytes)
    };

    db.execute(&insert_params(
        include_str!("../../../sql/generate_session.sql"),
        &[("account_id", &account_id), ("token", &token)],
    ))
    .await?;

    Ok(token)
}
