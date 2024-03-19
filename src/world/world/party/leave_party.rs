use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{PartyCloseServerPacket, PartyRemoveServerPacket},
        PacketAction, PacketFamily,
    },
};

use super::super::World;

impl World {
    pub fn leave_party(&mut self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let party = match self
            .parties
            .iter_mut()
            .find(|p| p.members.contains(&player_id))
        {
            Some(party) => party,
            None => return,
        };

        party.members.retain(|&id| id != player_id);

        player.send(
            PacketAction::Close,
            PacketFamily::Party,
            &PartyCloseServerPacket::new(),
        );

        let packet = PartyRemoveServerPacket { player_id };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing PartyRemoveServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for member_id in &party.members {
            let member = match self.players.get(member_id) {
                Some(member) => member,
                None => continue,
            };

            member.send_buf(PacketAction::Remove, PacketFamily::Party, buf.clone());
        }
    }
}
