use eo::{
    data::{i32, EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn update_party_hp(&self, player_id: EOShort, hp_percentage: i32) {
        if let Some(party) = self.get_player_party(player_id) {
            let mut builder = StreamBuilder::new();
            builder.add_short(player_id);
            builder.add_char(hp_percentage);

            let buf = builder.get();

            for member_id in &party.members {
                let member = match self.players.get(member_id) {
                    Some(member) => member,
                    None => continue,
                };

                member.send(PacketAction::Agree, PacketFamily::Party, buf.clone());
            }
        }
    }
}
