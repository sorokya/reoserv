use eolib::protocol::{
    map::MapTimedEffect,
    net::{
        PacketAction, PacketFamily,
        server::{
            EffectUseServerPacket, EffectUseServerPacketEffectData,
            EffectUseServerPacketEffectDataQuake, MapEffect,
        },
    },
};
use rand::RngExt;

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn timed_quake(&mut self) {
        if !matches!(
            self.file.timed_effect,
            MapTimedEffect::Quake1
                | MapTimedEffect::Quake2
                | MapTimedEffect::Quake3
                | MapTimedEffect::Quake4
        ) {
            return;
        }

        let config = match self.file.timed_effect {
            MapTimedEffect::Quake1 => &SETTINGS.load().map.quakes[0],
            MapTimedEffect::Quake2 => &SETTINGS.load().map.quakes[1],
            MapTimedEffect::Quake3 => &SETTINGS.load().map.quakes[2],
            MapTimedEffect::Quake4 => &SETTINGS.load().map.quakes[3],
            _ => return,
        };

        let mut rng = rand::rng();

        let rate = match self.quake_rate {
            Some(rate) => rate,
            None => {
                let rate = rng.random_range(config.min_ticks..=config.max_ticks);
                self.quake_rate = Some(rate);
                rate
            }
        };

        let quake_strength = match self.quake_strength {
            Some(strength) => strength,
            None => {
                let strength = rng.random_range(config.min_strength..=config.max_strength);
                self.quake_strength = Some(strength);
                strength
            }
        };

        self.quake_ticks += 1;
        if self.quake_ticks >= rate {
            self.send_packet_all(
                PacketAction::Use,
                PacketFamily::Effect,
                EffectUseServerPacket {
                    effect: MapEffect::Quake,
                    effect_data: Some(EffectUseServerPacketEffectData::Quake(
                        EffectUseServerPacketEffectDataQuake { quake_strength },
                    )),
                },
            );

            self.quake_rate = None;
            self.quake_strength = None;
            self.quake_ticks = 0;
        }
    }
}
