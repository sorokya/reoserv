use crate::db::{DbHandle, insert_params};

pub async fn guild_exists(db: &DbHandle, guild_tag: &str, guild_name: &str) -> bool {
    matches!(
        db.query_one(&insert_params(
            "SELECT id FROM `guilds` WHERE name = :name OR tag = :tag",
            &[("name", &guild_name), ("tag", &guild_tag)],
        ))
        .await,
        Ok(Some(_))
    )
}
