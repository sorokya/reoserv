use crate::{utils::get_distance, SETTINGS};

use super::super::Map;

impl Map {
    pub fn timed_auto_pickup(&mut self) {
        let mut matched: Vec<(i32, i32)> = Vec::new();

        for (item_index, item) in self.items.iter() {
            if let Some(player_id) = self
                .characters
                .iter()
                .filter(|(_, character)| {
                    let distance = get_distance(&item.coords, &character.coords);
                    !character.captcha_open
                        && character.auto_pickup_items.contains(&item.id)
                        && distance <= SETTINGS.world.drop_distance
                })
                .min_by(|(_, a), (_, b)| {
                    let distance_a = get_distance(&item.coords, &a.coords);
                    let distance_b = get_distance(&item.coords, &b.coords);
                    distance_a.cmp(&distance_b)
                })
                .map(|(player_id, _)| *player_id)
            {
                matched.push((*item_index, player_id));
            }
        }

        for (item_index, player_id) in matched {
            self.get_item(player_id, item_index);
        }
    }
}
