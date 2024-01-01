use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::PartyRemoveServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn disband_party(&mut self, leader_id: i32) {
        let party_index = match self.parties.iter().position(|p| p.leader == leader_id) {
            Some(index) => index,
            None => return,
        };

        let party = self.parties.remove(party_index);

        let packet = PartyRemoveServerPacket {
            player_id: leader_id,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize PartyRemoveServerPacket: {}", e);
            return;
        }

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
