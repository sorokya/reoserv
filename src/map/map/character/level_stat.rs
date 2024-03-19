use eolib::protocol::net::{
    client::StatId, server::StatSkillPlayerServerPacket, PacketAction, PacketFamily,
};

use super::super::Map;

impl Map {
    pub fn level_stat(&mut self, player_id: i32, stat_id: StatId) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                return;
            }
        };

        if character.stat_points == 0 {
            return;
        }

        match stat_id {
            StatId::Str => {
                character.base_strength += 1;
            }
            StatId::Int => {
                character.base_intelligence += 1;
            }
            StatId::Wis => {
                character.base_wisdom += 1;
            }
            StatId::Agi => {
                character.base_agility += 1;
            }
            StatId::Con => {
                character.base_constitution += 1;
            }
            StatId::Cha => {
                character.base_charisma += 1;
            }
            _ => return,
        }

        character.stat_points -= 1;

        character.calculate_stats();

        character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::StatSkill,
            &StatSkillPlayerServerPacket {
                stat_points: character.stat_points,
                stats: character.get_character_stats_update(),
            },
        );
    }
}
