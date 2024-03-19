use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::PartyAgreeServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub fn update_party_hp(&self, player_id: i32, hp_percentage: i32) {
        if let Some(party) = self.get_player_party(player_id) {
            let packet = PartyAgreeServerPacket {
                player_id,
                hp_percentage,
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize PartyAgreeServerPacket: {}", e);
                return;
            }

            let buf = writer.to_byte_array();

            for member_id in &party.members {
                let member = match self.players.get(member_id) {
                    Some(member) => member,
                    None => continue,
                };

                member.send_buf(PacketAction::Agree, PacketFamily::Party, buf.clone());
            }
        }
    }
}
