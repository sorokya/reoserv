use eo::data::EOInt;
use mysql_async::{prelude::*, Conn, TxOpts};

use crate::SETTINGS;

use super::Character;

impl Character {
    pub async fn create(
        &mut self,
        conn: &mut Conn,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let mut tx = conn.start_transaction(TxOpts::default()).await?;

        tx.exec_drop(
            include_str!("../sql/create_character.sql"),
            params! {
                "account_id" => &self.account_id,
                "name" => &self.name,
                "home" => &SETTINGS.new_character.home,
                "gender" => &(self.gender as u32),
                "race" => &(self.skin as u32),
                "hair_style" => &(self.hair_style as u32),
                "hair_color" => &(self.hair_color as u32),
                "bank_max" => &0_u32, // TODO: figure out bank max
            },
        )
        .await?;

        self.id = tx.last_insert_id().unwrap() as EOInt;

        tx.exec_drop(
            r"INSERT INTO `Paperdoll` (
                `character_id`
            ) VALUES (:character_id);",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.exec_drop(
            r"INSERT INTO `Position` (
                `character_id`,
                `map`,
                `x`,
                `y`,
                `direction`
            ) VALUES (
                :character_id,
                :map,
                :x,
                :y,
                :direction
            );",
            params! {
                "character_id" => &self.id,
                "map" => &SETTINGS.new_character.spawn_map,
                "x" => &SETTINGS.new_character.spawn_x,
                "y" => &SETTINGS.new_character.spawn_y,
                "direction" => &SETTINGS.new_character.spawn_direction,
            },
        )
        .await?;

        tx.exec_drop(
            r" INSERT INTO `Stats` (`character_id`)
            VALUES (:character_id);",
            params! {
                "character_id" => &self.id,
            },
        )
        .await?;

        tx.commit().await?;

        Ok(())
    }
}
