use std::mem;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::EffectPlayerServerPacket, PacketAction, PacketFamily},
};

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

        let packet = EffectPlayerServerPacket {
            player_id,
            effect_id,
        };

        let mut writer = EoWriter::with_capacity(mem::size_of::<EffectPlayerServerPacket>());
        packet.serialize(&mut writer);
        let buf = writer.to_byte_array();

        for (id, character) in self.characters.iter() {
            if let Some(player) = character.player.as_ref() {
                if &player_id != id {
                    player.send(PacketAction::Player, PacketFamily::Effect, buf.clone());
                }
            }
        }
    }
}
