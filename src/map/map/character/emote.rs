use eolib::protocol::{
    net::{server::EmotePlayerServerPacket, PacketAction, PacketFamily},
    Emote,
};

use super::super::Map;

impl Map {
    pub fn emote(&self, player_id: i32, emote: Emote) {
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
            PacketFamily::Emote,
            &EmotePlayerServerPacket { player_id, emote },
        );
    }
}
