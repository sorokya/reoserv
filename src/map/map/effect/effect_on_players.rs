use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{EffectPlayerServerPacket, PlayerEffect},
        PacketAction, PacketFamily,
    },
};

use crate::utils::in_client_range;

use super::super::Map;

impl Map {
    pub fn effect_on_players(&mut self, player_ids: &[i32], effect_id: i32) {
        let packet = EffectPlayerServerPacket {
            effects: player_ids
                .iter()
                .map(|id| PlayerEffect {
                    player_id: *id,
                    effect_id,
                })
                .collect::<Vec<_>>(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing EffectPlayerServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for character in self.characters.values() {
            if player_ids.iter().any(|id| {
                let target_character = match self.characters.get(id) {
                    Some(character) => character,
                    None => return false,
                };

                !target_character.hidden
                    && in_client_range(&character.coords, &target_character.coords)
            }) {
                let player = match character.player {
                    Some(ref player) => player,
                    None => continue,
                };

                player.send_buf(PacketAction::Player, PacketFamily::Effect, buf.clone());
            }
        }
    }
}
