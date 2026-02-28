use crate::{
    SETTINGS,
    db::{DbHandle, insert_params},
};

use eolib::protocol::AdminLevel;

use super::Character;

impl Character {
    pub async fn create(
        &mut self,
        db: &DbHandle,
    ) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
        let admin_level = if SETTINGS.server.auto_admin {
            match db.query_int("SELECT COUNT(1) FROM characters").await? {
                Some(0) => AdminLevel::HighGameMaster,
                _ => AdminLevel::Player,
            }
        } else {
            AdminLevel::Player
        };

        db.execute(&insert_params(
            include_str!("../sql/create_character.sql"),
            &[
                ("account_id", &self.account_id),
                ("name", &self.name),
                ("home", &SETTINGS.new_character.home),
                ("gender", &(i32::from(self.gender))),
                ("race", &self.skin),
                ("hair_style", &(self.hair_style as u32)),
                ("hair_color", &(self.hair_color as u32)),
                ("map", &SETTINGS.new_character.spawn_map),
                ("x", &SETTINGS.new_character.spawn_x),
                ("y", &SETTINGS.new_character.spawn_y),
                ("direction", &SETTINGS.new_character.spawn_direction),
                ("admin_level", &i32::from(admin_level)),
            ],
        ))
        .await?;

        self.id = db.get_last_insert_id().await.unwrap() as i32;

        Ok(())
    }
}
