use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{EffectAgreeServerPacket, TileEffect},
            PacketAction, PacketFamily,
        },
        Coords,
    },
};

use crate::utils::in_client_range;

use super::super::Map;

impl Map {
    pub fn effect_on_coords(&self, coords: &[Coords], effect_id: i32) {
        let packet = EffectAgreeServerPacket {
            effects: coords
                .iter()
                .map(|coords| TileEffect {
                    coords: *coords,
                    effect_id,
                })
                .collect::<Vec<_>>(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing EffectAgreeServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for character in self.characters.values() {
            if coords
                .iter()
                .any(|coords| in_client_range(&character.coords, coords))
            {
                let player = match character.player {
                    Some(ref player) => player,
                    None => continue,
                };

                player.send_buf(PacketAction::Agree, PacketFamily::Effect, buf.clone());
            }
        }
    }
}
