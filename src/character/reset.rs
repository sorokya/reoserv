use crate::SETTINGS;

use super::Character;

impl Character {
    pub fn reset(&mut self) {
        self.base_strength = 0;
        self.base_intelligence = 0;
        self.base_wisdom = 0;
        self.base_agility = 0;
        self.base_constitution = 0;
        self.base_charisma = 0;

        self.spells.clear();
        self.stat_points = self.level * SETTINGS.world.stat_points_per_level;
        self.skill_points = self.level * SETTINGS.world.skill_points_per_level;

        self.calculate_stats();
    }
}
