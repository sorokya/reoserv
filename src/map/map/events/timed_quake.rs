use eolib::{
    data::EoWriter,
    protocol::{
        map::MapTimedEffect,
        net::{PacketAction, PacketFamily},
    },
};
use rand::{thread_rng, Rng};

use crate::SETTINGS;

use super::super::Map;

const EFFECT_QUAKE: i32 = 1;

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
            MapTimedEffect::Quake1 => &SETTINGS.map.quakes[0],
            MapTimedEffect::Quake2 => &SETTINGS.map.quakes[1],
            MapTimedEffect::Quake3 => &SETTINGS.map.quakes[2],
            MapTimedEffect::Quake4 => &SETTINGS.map.quakes[3],
            _ => return,
        };

        let mut rng = thread_rng();

        let rate = match self.quake_rate {
            Some(rate) => rate,
            None => {
                let rate = rng.gen_range(config.min_ticks..=config.max_ticks);
                self.quake_rate = Some(rate);
                rate
            }
        };

        let strength = match self.quake_strength {
            Some(strength) => strength,
            None => {
                let strength = rng.gen_range(config.min_strength..=config.max_strength);
                self.quake_strength = Some(strength);
                strength
            }
        };

        self.quake_ticks += 1;
        if self.quake_ticks >= rate {
            let mut writer = EoWriter::new();
            writer.add_char(EFFECT_QUAKE);
            writer.add_char(strength as i32);

            let buf = writer.to_byte_array();

            for character in self.characters.values() {
                character.player.as_ref().unwrap().send(
                    PacketAction::Use,
                    PacketFamily::Effect,
                    buf.clone(),
                );
            }

            self.quake_rate = None;
            self.quake_strength = None;
            self.quake_ticks = 0;
        }
    }
}
