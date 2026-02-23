use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn timed_cleanup(&mut self) {
        if SETTINGS.map.max_items <= 0 || self.items.len() <= SETTINGS.map.max_items as usize {
            return;
        }

        let mut item_indexes_and_timestamps: Vec<(i32, i64)> = self
            .items
            .iter()
            .map(|(index, item)| (*index, item.drop_time))
            .collect();

        item_indexes_and_timestamps.sort_by_key(|&(_, drop_time)| drop_time);
        item_indexes_and_timestamps.truncate(self.items.len() - SETTINGS.map.max_items as usize);

        for (index, _) in item_indexes_and_timestamps {
            self.remove_item(index);
        }
    }
}
