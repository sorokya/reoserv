use eolib::protocol::net::{server::TalkPlayerServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn send_chat_message(&self, player_id: i32, message: String) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.hidden {
            return;
        }

        self.send_packet_near_player(
            player_id,
            PacketAction::Player,
            PacketFamily::Talk,
            &TalkPlayerServerPacket { player_id, message },
        );
    }
}
