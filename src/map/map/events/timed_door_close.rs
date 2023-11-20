use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn timed_door_close(&mut self) {
        for door in self.doors.iter_mut() {
            if !door.open {
                continue;
            }

            door.open_ticks += 1;

            if door.open_ticks >= SETTINGS.map.door_close_rate {
                door.open = false;
                door.open_ticks = 0;
            }
        }
    }
}
