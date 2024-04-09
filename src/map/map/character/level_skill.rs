use eolib::protocol::net::{server::StatSkillAcceptServerPacket, PacketAction, PacketFamily};

use super::super::Map;

impl Map {
    pub fn level_skill(&mut self, player_id: i32, skill_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                return;
            }
        };

        if character.skill_points == 0 {
            return;
        }

        let skill = match character.spells.iter_mut().find(|s| s.id == skill_id) {
            Some(skill) => skill,
            None => return,
        };

        skill.level += 1;
        character.skill_points -= 1;

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Accept,
                PacketFamily::StatSkill,
                &StatSkillAcceptServerPacket {
                    skill_points: character.skill_points,
                    spell: skill.to_owned(),
                },
            );
        }
    }
}
