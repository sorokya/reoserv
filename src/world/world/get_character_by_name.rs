use crate::{character::Character, errors::{DataNotFoundError, CharacterNotFoundError}};

use super::World;

impl World {
    pub async fn get_character_by_name(
        &self,
        name: &str,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Send + Sync>> {
        if let Some(player_id) = self.characters.get(name) {
            if let Some(player) = self.players.get(player_id) {
                // Safe to assume this will work if we got this far
                let character = player.get_character().await.unwrap();
                Ok(character)
            } else {
                Err(Box::new(DataNotFoundError::new(
                    "Player".to_string(),
                    *player_id,
                )))
            }
        } else {
            Err(Box::new(CharacterNotFoundError::new(name.to_string())))
        }
    }
}