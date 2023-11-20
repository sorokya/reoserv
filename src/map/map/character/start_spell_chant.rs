use chrono::Utc;
use eo::{
    data::{EOShort, EOThree},
    protocol::{server::spell::Request, PacketAction, PacketFamily},
};

use crate::character::SpellState;

use super::super::Map;

impl Map {
    pub fn start_spell_chant(&mut self, player_id: EOShort, spell_id: EOShort, timestamp: EOThree) {
        if spell_id == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.has_spell(spell_id) {
            return;
        }

        character.spell_state = SpellState::Requested {
            spell_id,
            timestamp,
            cast_time: Utc::now(),
        };

        if character.hidden {
            return;
        }

        let packet = Request {
            player_id,
            spell_id,
        };

        self.send_packet_near_player(
            player_id,
            PacketAction::Request,
            PacketFamily::Spell,
            packet,
        );
    }
}
