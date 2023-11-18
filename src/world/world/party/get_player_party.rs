use eo::data::EOShort;

use crate::world::Party;

use super::super::World;

impl World {
    pub fn get_player_party(&self, player_id: EOShort) -> Option<Party> {
        match self.parties.iter().find(|p| p.members.contains(&player_id)) {
            Some(party) => Some(party.to_owned()),
            None => None,
        }
    }
}
