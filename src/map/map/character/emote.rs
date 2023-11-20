use eo::{
    data::EOShort,
    protocol::{server::emote, Emote, PacketAction, PacketFamily},
};

use super::super::Map;

impl Map {
    pub fn emote(&self, target_player_id: EOShort, emote: Emote) {
        let character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if character.hidden {
            return;
        }

        let packet = emote::Player {
            player_id: target_player_id,
            emote,
        };

        self.send_packet_near_player(
            target_player_id,
            PacketAction::Player,
            PacketFamily::Emote,
            packet,
        );
    }
}
