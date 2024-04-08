use eolib::protocol::net::{server::PlayersPongServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn find_player(&self, player_id: i32, name: String) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        if self
            .characters
            .iter()
            .any(|(_, character)| character.name == name)
        {
            player.send(
                PacketAction::Pong,
                PacketFamily::Players,
                &PlayersPongServerPacket { name },
            );
        } else {
            self.world.find_player(player_id, name);
        }
    }
}
