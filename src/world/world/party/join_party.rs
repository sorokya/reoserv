use eo::data::EOShort;

use super::super::World;

impl World {
    pub fn join_party(&mut self, player_id: EOShort, party_member_id: EOShort) {
        let party = match self
            .parties
            .iter_mut()
            .find(|p| p.members.contains(&party_member_id))
        {
            Some(party) => party,
            None => return,
        };

        party.members.push(player_id);
    }
}
