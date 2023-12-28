use crate::world::Party;

use super::super::World;

impl World {
    pub fn get_player_party(&self, player_id: i32) -> Option<Party> {
        self.parties
            .iter()
            .find(|p| p.members.contains(&player_id))
            .map(|p| p.to_owned())
    }
}
