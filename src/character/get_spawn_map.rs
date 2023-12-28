use eo::data::i32;

use crate::{INN_DB, SETTINGS};

use super::Character;

impl Character {
    pub fn get_spawn_map(&self) -> i32 {
        match INN_DB.inns.iter().find(|inn| inn.name == self.home) {
            Some(inn) => {
                if inn.alt_spawn_enabled == 1 && self.level > 0 {
                    inn.alt_spawn_map
                } else {
                    inn.spawn_map
                }
            }
            None => SETTINGS.rescue.map,
        }
    }
}
