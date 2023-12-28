use eolib::{data::EoWriter, protocol::net::{PacketAction, PacketFamily}};

use super::super::World;

impl World {
    pub fn disband_party(&mut self, leader_id: i32) {
        let party_index = match self.parties.iter().position(|p| p.leader == leader_id) {
            Some(index) => index,
            None => return,
        };

        let party = self.parties.remove(party_index);

        let mut writer = EoWriter::new();
        writer.add_short(leader_id);

        let buf = writer.to_byte_array();

        for member_id in &party.members {
            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send(PacketAction::Remove, PacketFamily::Party, buf.clone());
        }
    }
}
