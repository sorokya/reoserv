use eo::data::i32;

use super::super::World;

impl World {
    pub fn player_in_party(&self, player_id: i32) -> bool {
        self.parties.iter().any(|p| p.members.contains(&player_id))
    }
}
