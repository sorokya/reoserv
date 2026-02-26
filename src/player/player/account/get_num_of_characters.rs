use crate::db::{insert_params, DbHandle};

pub async fn get_num_of_characters(
    db: &DbHandle,
    account_id: i32,
) -> Result<usize, Box<dyn std::error::Error + Send + Sync>> {
    match db
        .query_int(&insert_params(
            r"SELECT COUNT(id) FROM `characters` WHERE `account_id` = :account_id",
            &[("account_id", &account_id)],
        ))
        .await?
    {
        Some(count) => Ok(count as usize),
        _ => Ok(0),
    }
}
