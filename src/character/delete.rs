use crate::db::{DbHandle, insert_params};

use super::Character;

impl Character {
    pub async fn delete(&self, db: &DbHandle) -> anyhow::Result<()> {
        db.execute(&insert_params(
            r"DELETE FROM `characters` WHERE `id` = :character_id;",
            &[("character_id", &self.id)],
        ))
        .await
    }
}
