use eolib::protocol::{
    net::{server::FacePlayerServerPacket, PacketAction, PacketFamily},
    Direction,
};

use super::super::Map;

impl Map {
    pub fn face(&mut self, player_id: i32, direction: Direction) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.direction = direction;

        if character.hidden {
            return;
        }

        let packet = FacePlayerServerPacket {
            player_id,
            direction,
        };

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Face, packet);
    }
}
