use eo::{
    data::i32,
    protocol::{server::face, Direction, PacketAction, PacketFamily},
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

        let packet = face::Player {
            player_id,
            direction,
        };

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Face, packet);
    }
}
