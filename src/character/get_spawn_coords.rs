use eolib::protocol::Coords;

use crate::{INN_DB, SETTINGS};

use super::Character;

impl Character {
    pub fn get_spawn_coords(&self) -> Coords {
        match INN_DB.inns.iter().find(|inn| inn.name == self.home) {
            Some(inn) => {
                if inn.alternate_spawn_enabled && self.level > 0 {
                    Coords {
                        x: inn.alternate_spawn_x,
                        y: inn.alternate_spawn_y,
                    }
                } else {
                    Coords {
                        x: inn.spawn_x,
                        y: inn.spawn_y,
                    }
                }
            }
            None => Coords {
                x: SETTINGS.rescue.x,
                y: SETTINGS.rescue.y,
            },
        }
    }
}
