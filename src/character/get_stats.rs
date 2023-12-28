use eo::{
    data::i32,
    protocol::{
        CharacterBaseStats, CharacterBaseStats2, CharacterSecondaryStats, CharacterStats1,
        CharacterStats2, CharacterStats3, ItemCharacterStats,
    },
};

use super::Character;

impl Character {
    pub fn get_character_stats_1(&self) -> CharacterStats1 {
        CharacterStats1 {
            hp: self.hp,
            max_hp: self.max_hp,
            tp: self.tp,
            max_tp: self.max_tp,
            max_sp: self.max_sp,
            stat_points: self.stat_points,
            skill_points: self.skill_points,
            secondary: CharacterSecondaryStats {
                mindam: self.min_damage,
                maxdam: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
            base: CharacterBaseStats {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
        }
    }

    pub fn get_character_stats_2(&self) -> CharacterStats2 {
        CharacterStats2 {
            hp: self.hp,
            max_hp: self.max_hp,
            tp: self.tp,
            max_tp: self.max_tp,
            max_sp: self.max_sp,
            stat_points: self.stat_points,
            skill_points: self.skill_points,
            karma: self.karma,
            secondary: CharacterSecondaryStats {
                mindam: self.min_damage,
                maxdam: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
            base: CharacterBaseStats2 {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
        }
    }

    pub fn get_character_stats_3(&self) -> CharacterStats3 {
        CharacterStats3 {
            base: CharacterBaseStats {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
            max_hp: self.max_hp,
            max_tp: self.max_tp,
            max_sp: self.max_sp,
            max_weight: self.max_weight as i32,
            secondary: CharacterSecondaryStats {
                mindam: self.min_damage,
                maxdam: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
        }
    }

    pub fn get_item_character_stats(&self) -> ItemCharacterStats {
        ItemCharacterStats {
            max_hp: self.max_hp,
            max_tp: self.max_tp,
            base: CharacterBaseStats {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
            secondary: CharacterSecondaryStats {
                mindam: self.min_damage,
                maxdam: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
        }
    }
}
