use eo::data::EOShort;

use super::super::World;

impl World {
    pub fn player_in_party(&self, player_id: EOShort) -> bool {
        self.parties.iter().any(|p| p.members.contains(&player_id))
    }
}
