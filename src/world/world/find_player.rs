use eolib::protocol::net::{
    server::{PlayersNet242ServerPacket, PlayersPingServerPacket},
    PacketAction, PacketFamily,
};

use super::World;

impl World {
    pub fn find_player(&self, player_id: i32, name: String) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        if self.characters.contains_key(&name) {
            player.send(
                PacketAction::Net242,
                PacketFamily::Players,
                &PlayersNet242ServerPacket { name },
            );
        } else {
            player.send(
                PacketAction::Ping,
                PacketFamily::Players,
                &PlayersPingServerPacket { name },
            );
        }
    }
}
