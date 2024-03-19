use eolib::protocol::{
    net::{server::EmotePlayerServerPacket, PacketAction, PacketFamily},
    Emote,
};

use super::super::Map;

impl Map {
    pub fn emote(&self, target_player_id: i32, emote: Emote) {
        let character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if character.hidden {
            return;
        }

        self.send_packet_near_player(
            target_player_id,
            PacketAction::Player,
            PacketFamily::Emote,
            &EmotePlayerServerPacket {
                player_id: target_player_id,
                emote,
            },
        );
    }
}
