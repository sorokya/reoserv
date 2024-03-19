use eolib::protocol::net::{server::EffectPlayerServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn play_effect(&mut self, player_id: i32, effect_id: i32) {
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
            PacketFamily::Effect,
            &EffectPlayerServerPacket {
                player_id,
                effect_id,
            },
        );
    }
}
