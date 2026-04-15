use std::cmp;

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn timed_usage(&mut self) {
        for character in self.characters.values_mut() {
            character.usage_ticks = cmp::max(0, character.usage_ticks - 1);
            if character.usage_ticks == 0 {
                character.usage += 1;
                character.usage_ticks = SETTINGS.world.usage_rate;
            }
        }
    }
}
