use crate::db::{insert_params, DbHandle};

pub async fn account_exists(
    db: &DbHandle,
    name: &str,
) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
    match db
        .query_one(&insert_params(
            r"SELECT id FROM `accounts` WHERE `name` = :name",
            &[("name", &name)],
        ))
        .await?
    {
        Some(_) => Ok(true),
        _ => Ok(false),
    }
}
