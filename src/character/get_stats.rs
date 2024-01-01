use eolib::protocol::net::server::{
    CharacterBaseStats, CharacterBaseStatsWelcome, CharacterSecondaryStats,
    CharacterStatsEquipmentChange, CharacterStatsReset, CharacterStatsUpdate,
    CharacterStatsWelcome,
};

use super::Character;

impl Character {
    pub fn get_character_stats_reset(&self) -> CharacterStatsReset {
        CharacterStatsReset {
            hp: self.hp,
            max_hp: self.max_hp,
            tp: self.tp,
            max_tp: self.max_tp,
            max_sp: self.max_sp,
            stat_points: self.stat_points,
            skill_points: self.skill_points,
            secondary: CharacterSecondaryStats {
                min_damage: self.min_damage,
                max_damage: self.max_damage,
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

    pub fn get_character_stats_welcome(&self) -> CharacterStatsWelcome {
        CharacterStatsWelcome {
            hp: self.hp,
            max_hp: self.max_hp,
            tp: self.tp,
            max_tp: self.max_tp,
            max_sp: self.max_sp,
            stat_points: self.stat_points,
            skill_points: self.skill_points,
            karma: self.karma,
            secondary: CharacterSecondaryStats {
                min_damage: self.min_damage,
                max_damage: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
            base: CharacterBaseStatsWelcome {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
        }
    }

    pub fn get_character_stats_update(&self) -> CharacterStatsUpdate {
        CharacterStatsUpdate {
            base_stats: CharacterBaseStats {
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
            max_weight: self.max_weight,
            secondary_stats: CharacterSecondaryStats {
                min_damage: self.min_damage,
                max_damage: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
        }
    }

    pub fn get_character_stats_equipment_change(&self) -> CharacterStatsEquipmentChange {
        CharacterStatsEquipmentChange {
            max_hp: self.max_hp,
            max_tp: self.max_tp,
            base_stats: CharacterBaseStats {
                str: self.adj_strength,
                intl: self.adj_intelligence,
                wis: self.adj_wisdom,
                agi: self.adj_agility,
                con: self.adj_constitution,
                cha: self.adj_charisma,
            },
            secondary_stats: CharacterSecondaryStats {
                min_damage: self.min_damage,
                max_damage: self.max_damage,
                accuracy: self.accuracy,
                evade: self.evasion,
                armor: self.armor,
            },
        }
    }
}
