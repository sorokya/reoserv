use eo::data::i32;

use super::super::World;

impl World {
    pub fn remove_party_member(&mut self, player_id: i32, target_player_id: i32) {
        let party = match self
            .parties
            .iter_mut()
            .find(|p| p.members.contains(&player_id))
        {
            Some(party) => party,
            None => return,
        };

        let leader_id = party.leader;
        if leader_id == player_id && player_id == target_player_id || party.members.len() == 2 {
            self.disband_party(leader_id);
        } else {
            self.leave_party(target_player_id);
        }
    }
}
