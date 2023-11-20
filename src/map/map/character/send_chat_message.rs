use eo::{
    data::EOShort,
    protocol::{server::talk, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        let character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if character.hidden {
            return;
        }

        let packet = talk::Player {
            player_id: target_player_id,
            message,
        };

        self.send_packet_near_player(
            target_player_id,
            PacketAction::Player,
            PacketFamily::Talk,
            packet,
        );
    }
}
