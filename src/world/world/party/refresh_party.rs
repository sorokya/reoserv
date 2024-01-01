use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::PartyListServerPacket, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub async fn refresh_party(&self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let party = match self.get_player_party(player_id) {
            Some(party) => party,
            None => return,
        };

        let packet = PartyListServerPacket {
            members: self.get_party_members(&party).await,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize party list packet: {}", e);
            return;
        }

        player.send(
            PacketAction::List,
            PacketFamily::Party,
            writer.to_byte_array(),
        );
    }
}
