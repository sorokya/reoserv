use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn timed_cleanup(&mut self) {
        if SETTINGS.load().map.max_items <= 0 || self.items.len() <= SETTINGS.load().map.max_items as usize {
            return;
        }

        for i in (0..self.items.len() - SETTINGS.load().map.max_items as usize).rev() {
            self.remove_item(self.items[i].index);
        }
    }
}
