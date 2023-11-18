use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn broadcast_party_message(&self, player_id: EOShort, message: String) {
        let party = match self.parties.iter().find(|p| p.members.contains(&player_id)) {
            Some(party) => party,
            None => return,
        };

        let mut builder = StreamBuilder::new();
        builder.add_short(player_id);
        builder.add_string(&message);

        let buf = builder.get();

        for member_id in &party.members {
            if *member_id == player_id {
                continue;
            }

            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send(PacketAction::Open, PacketFamily::Talk, buf.clone());
        }
    }
}
