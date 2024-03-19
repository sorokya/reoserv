use eolib::protocol::net::{
    server::{SitReplyServerPacket, SitState},
    PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn sit(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character {}", player_id);
                return;
            }
        };

        if character.sit_state != SitState::Stand {
            return;
        }

        character.sit_state = SitState::Floor;

        let packet = SitReplyServerPacket {
            player_id,
            coords: character.coords,
            direction: character.direction,
        };

        character
            .player
            .as_ref()
            .unwrap()
            .send(PacketAction::Reply, PacketFamily::Sit, &packet);

        if !character.hidden {
            self.send_packet_near_player(
                player_id,
                PacketAction::Player,
                PacketFamily::Sit,
                &packet,
            );
        }
    }
}
