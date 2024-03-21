use eolib::protocol::net::{server::TalkPlayerServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn player_chat(&self, player_id: i32, message: &str) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        self.send_packet_near(
            &character.coords,
            PacketAction::Player,
            PacketFamily::Talk,
            TalkPlayerServerPacket {
                player_id,
                message: message.to_owned(),
            },
        );
    }
}
