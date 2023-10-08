use mysql_async::{prelude::*, Conn, TxOpts};

use super::Character;

impl Character {
    pub async fn delete(
        &self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            r"DELETE FROM `Stats` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Spell` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Position` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Paperdoll` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Inventory` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Bank` WHERE `character_id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"DELETE FROM `Character` WHERE `id` = :character_id;",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.commit().await?;
        Ok(())
    }
}
