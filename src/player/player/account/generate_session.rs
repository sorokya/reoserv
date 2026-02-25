use rand::{rngs::OsRng, RngCore};

use crate::db::{insert_params, DbHandle};

pub async fn generate_session(db: &DbHandle, account_id: i32) -> anyhow::Result<String> {
    let mut bytes = [0u8; 32];
    OsRng.fill_bytes(&mut bytes);

    let token = hex::encode(bytes);

    db.execute(&insert_params(
        include_str!("../../../sql/generate_session.sql"),
        &[("account_id", &account_id), ("token", &token)],
    ))
    .await?;

    Ok(token)
}
