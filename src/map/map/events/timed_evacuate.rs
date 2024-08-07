use eolib::protocol::{
    net::{
        server::{MusicPlayerServerPacket, TalkServerServerPacket},
        PacketAction, PacketFamily,
    },
    AdminLevel, Coords,
};

use crate::{LANG, SETTINGS};

use super::super::Map;

impl Map {
    pub fn timed_evacuate(&mut self) {
        let seconds = match self.evacuate_ticks {
            Some(ref ticks) => ticks.to_owned(),
            None => return,
        };

        let num_steps = (SETTINGS.evacuate.timer_seconds as f32
            / SETTINGS.evacuate.timer_step as f32)
            .ceil() as usize;

        let mut steps = Vec::new();

        if num_steps > 1 {
            for i in 2..=num_steps {
                steps.push(SETTINGS.evacuate.timer_step * i as i32);
            }
        }

        if steps.contains(&seconds) {
            self.send_evac_warning(&LANG.evacuate_warning, seconds);
        }

        if seconds == SETTINGS.evacuate.timer_step {
            self.send_evac_warning(&LANG.evacuate_last_warning, seconds);
        }

        if seconds == 0 {
            for character in self.characters.values() {
                if character.admin_level != AdminLevel::Player {
                    continue;
                }

                let player = match character.player {
                    Some(ref player) => player,
                    None => continue,
                };

                player.request_warp(
                    SETTINGS.jail.map,
                    Coords {
                        x: SETTINGS.jail.x,
                        y: SETTINGS.jail.y,
                    },
                    false,
                    None,
                );
            }

            self.evacuate_ticks = None;
        } else if let Some(ticks) = self.evacuate_ticks.as_mut() {
            *ticks -= 1;
        }
    }

    fn send_evac_warning(&self, template: &str, seconds: i32) {
        self.send_packet_all(
            PacketAction::Server,
            PacketFamily::Talk,
            TalkServerServerPacket {
                message: get_lang_string!(template, seconds = seconds),
            },
        );

        self.send_packet_all(
            PacketAction::Player,
            PacketFamily::Music,
            MusicPlayerServerPacket {
                sound_id: SETTINGS.evacuate.sfx_id,
            },
        );
    }
}
