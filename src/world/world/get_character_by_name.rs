use crate::{
    character::Character,
    errors::{CharacterNotFoundError, DataNotFoundError},
};

use super::World;

impl World {
    pub async fn get_character_by_name(
        &self,
        name: &str,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Sync + Send>> {
        let player_id = self.characters.get(name);

        if player_id.is_none() {
            return Err(Box::new(CharacterNotFoundError::new(name.to_string())));
        }

        let player_id = player_id.unwrap();

        let player = self.players.get(player_id);

        if player.is_none() {
            return Err(Box::new(DataNotFoundError::new(
                "Player".to_string(),
                *player_id,
            )));
        }

        let player = player.unwrap();

        // Safe to assume this will work if we got this far
        let character = player.get_character().await.unwrap();
        Ok(character)
    }
}
