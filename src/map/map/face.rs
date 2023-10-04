use eo::{
    data::EOShort,
    protocol::{server::face, Direction, PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub fn face(&mut self, target_player_id: EOShort, direction: Direction) {
        {
            let mut target = self.characters.get_mut(&target_player_id).unwrap();
            target.direction = direction;
        }

        let packet = face::Player {
            player_id: target_player_id,
            direction,
        };

        self.send_packet_near_player(
            target_player_id,
            PacketAction::Player,
            PacketFamily::Face,
            packet,
        );
    }
}
