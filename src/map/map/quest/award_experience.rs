use eolib::protocol::net::{server::RecoverReplyServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn award_experience(&mut self, player_id: i32, amount: i32) {
        let (leveled_up, level, experience, _) = self.give_experience(player_id, amount);

        if experience == 0 {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player {
            Some(ref player) => player,
            None => return,
        };

        player.send(
            PacketAction::Reply,
            PacketFamily::Recover,
            &RecoverReplyServerPacket {
                experience,
                karma: character.karma,
                level_up: if leveled_up { Some(level) } else { None },
                stat_points: if leveled_up {
                    Some(character.stat_points)
                } else {
                    None
                },
                skill_points: if leveled_up {
                    Some(character.skill_points)
                } else {
                    None
                },
            },
        )
    }
}
