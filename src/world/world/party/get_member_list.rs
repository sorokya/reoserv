use eolib::protocol::net::server::PartyMember;

use crate::world::Party;

use super::super::World;

impl World {
    pub async fn get_party_members(&self, party: &Party) -> Vec<PartyMember> {
        let mut members = Vec::new();
        let leader_id = party.leader;

        for member_id in &party.members {
            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            let character = match member.get_character().await {
                Ok(character) => character,
                Err(_) => continue,
            };

            members.push(PartyMember {
                player_id: *member_id,
                leader: *member_id == leader_id,
                level: character.level,
                hp_percentage: character.get_hp_percentage(),
                name: character.name.clone(),
            });
        }

        members
    }
}
