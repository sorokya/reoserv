use crate::db::{DbHandle, insert_params};

pub async fn get_guild_ranks(db: &DbHandle, tag: &str) -> Vec<String> {
    match db
        .query_map(
            &insert_params(
                include_str!("../sql/get_guild_ranks.sql"),
                &[("tag", &tag.to_string())],
            ),
            |row| row.get_string(0).unwrap(),
        )
        .await
    {
        Ok(ranks) => ranks,
        Err(e) => {
            tracing::error!("Error getting guild ranks: {}", e);
            vec!["".to_owned(); 9]
        }
    }
}
