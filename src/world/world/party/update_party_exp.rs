use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{PartyExpShare, PartyTargetGroupServerPacket},
        PacketAction, PacketFamily,
    },
};

use super::super::World;

impl World {
    pub fn update_party_exp(&self, player_id: i32, exp_gains: Vec<PartyExpShare>) {
        if let Some(party) = self.get_player_party(player_id) {
            let packet = PartyTargetGroupServerPacket { gains: exp_gains };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize PartyTargetGroupServerPacket: {}", e);
                return;
            }

            let buf = writer.to_byte_array();

            for member_id in &party.members {
                let member = match self.players.get(member_id) {
                    Some(member) => member,
                    None => continue,
                };

                member.send_buf(PacketAction::TargetGroup, PacketFamily::Party, buf.clone());
            }
        }
    }
}
