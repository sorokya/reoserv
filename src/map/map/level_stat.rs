use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::statskill::Player, PacketAction, PacketFamily, StatId},
};

use super::Map;

impl Map {
    pub fn level_stat(&mut self, player_id: EOShort, stat_id: StatId) {
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
        }

        character.stat_points -= 1;

        character.calculate_stats();

        let reply = Player {
            stat_points: character.stat_points,
            stats: character.get_character_stats_3(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        let _ = character.player.as_ref().unwrap().send(
            PacketAction::Player,
            PacketFamily::StatSkill,
            builder.get(),
        );
    }
}
