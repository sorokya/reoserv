use eolib::{data::EoWriter, protocol::net::{PacketAction, PacketFamily}};

use super::super::World;

impl World {
    pub fn broadcast_party_message(&self, player_id: i32, message: String) {
        let party = match self.parties.iter().find(|p| p.members.contains(&player_id)) {
            Some(party) => party,
            None => return,
        };

        let mut writer = EoWriter::new();
        writer.add_short(player_id);
        writer.add_string(&message);

        let buf = writer.to_byte_array();

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
