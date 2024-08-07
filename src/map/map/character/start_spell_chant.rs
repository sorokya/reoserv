use eolib::protocol::net::{server::SpellRequestServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn start_spell_chant(&mut self, player_id: i32, spell_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.has_spell(spell_id) {
            return;
        }

        if character.hidden {
            return;
        }

        self.send_packet_near_player(
            player_id,
            PacketAction::Request,
            PacketFamily::Spell,
            &SpellRequestServerPacket {
                player_id,
                spell_id,
            },
        );
    }
}
