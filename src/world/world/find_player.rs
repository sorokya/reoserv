use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{PlayersNet242ServerPacket, PlayersPingServerPacket},
        PacketAction, PacketFamily,
    },
};

use super::World;

impl World {
    pub fn find_player(&self, player_id: i32, name: String) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        if self.characters.contains_key(&name) {
            let packet = PlayersNet242ServerPacket { name };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing PlayersNet242ServerPacket: {}", e);
                return;
            }

            player.send(
                PacketAction::Net242,
                PacketFamily::Players,
                writer.to_byte_array(),
            );
        } else {
            let packet = PlayersPingServerPacket { name };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing PlayersPingServerPacket: {}", e);
                return;
            }

            player.send(
                PacketAction::Ping,
                PacketFamily::Players,
                writer.to_byte_array(),
            );
        }
    }
}
