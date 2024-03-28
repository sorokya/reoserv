use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            EffectUseServerPacket, EffectUseServerPacketEffectData,
            EffectUseServerPacketEffectDataQuake, MapEffect,
        },
        PacketAction, PacketFamily,
    },
};

use super::super::Map;

impl Map {
    pub fn quake(&self, magnitude: i32) {
        let magnitude = cmp::max(1, cmp::min(8, magnitude));

        let packet = EffectUseServerPacket {
            effect: MapEffect::Quake,
            effect_data: Some(EffectUseServerPacketEffectData::Quake(
                EffectUseServerPacketEffectDataQuake {
                    quake_strength: magnitude,
                },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize EffectUseServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for character in self.characters.values() {
            let player = match character.player {
                Some(ref player) => player,
                None => continue,
            };

            player.send_buf(PacketAction::Use, PacketFamily::Effect, buf.clone());
        }
    }
}
