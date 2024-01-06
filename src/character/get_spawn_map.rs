use crate::{INN_DB, SETTINGS};

use super::Character;

impl Character {
    pub fn get_spawn_map(&self) -> i32 {
        match INN_DB.inns.iter().find(|inn| inn.name == self.home) {
            Some(inn) => {
                if inn.alternate_spawn_enabled && self.level > 0 {
                    inn.alternate_spawn_map
                } else {
                    inn.spawn_map
                }
            }
            None => SETTINGS.rescue.map,
        }
    }
}
