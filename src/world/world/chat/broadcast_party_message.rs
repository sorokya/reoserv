use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::TalkOpenServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn broadcast_party_message(&self, player_id: i32, message: String) {
        let party = match self.parties.iter().find(|p| p.members.contains(&player_id)) {
            Some(party) => party,
            None => return,
        };

        let packet = TalkOpenServerPacket { player_id, message };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize TalkOpenServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for member_id in &party.members {
            if *member_id == player_id {
                continue;
            }

            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send_buf(PacketAction::Open, PacketFamily::Talk, buf.clone());
        }
    }
}
