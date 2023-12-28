use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn give_experience(
        &mut self,
        player_id: i32,
        experience: i32,
    ) -> (bool, i32, i32) {
        match self.characters.get_mut(&player_id) {
            Some(character) => {
                let experience = experience * SETTINGS.world.exp_multiplier;
                let leveled_up = character.add_experience(experience);
                (leveled_up, character.experience, experience)
            }
            None => (false, 0, 0),
        }
    }
}
