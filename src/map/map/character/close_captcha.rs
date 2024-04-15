use eolib::protocol::net::{
    server::{ItemAcceptServerPacket, RecoverReplyServerPacket},
    PacketAction, PacketFamily,
};

use crate::deep::{CaptchaCloseServerPacket, FAMILY_CAPTCHA};

use super::super::Map;

impl Map {
    pub fn close_captcha(&mut self, player_id: i32, experience: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.is_deep {
            return;
        }

        let leveled_up = character.add_experience(experience);
        character.captcha_open = false;

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if let Some(player) = &character.player {
            player.send(
                PacketAction::Close,
                PacketFamily::Unrecognized(FAMILY_CAPTCHA),
                &CaptchaCloseServerPacket {
                    experience: character.experience,
                },
            );

            if leveled_up {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Recover,
                    &RecoverReplyServerPacket {
                        experience,
                        karma: character.karma,
                        level_up: Some(character.level),
                        stat_points: Some(character.stat_points),
                        skill_points: Some(character.skill_points),
                    },
                );

                if leveled_up {
                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Accept,
                        PacketFamily::Item,
                        &ItemAcceptServerPacket { player_id },
                    );
                }
            }
        }
    }
}
