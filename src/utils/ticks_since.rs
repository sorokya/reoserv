use chrono::{DateTime, Utc};
use eo::data::EOInt;

use crate::SETTINGS;

pub fn ticks_since(when: &DateTime<Utc>) -> EOInt {
    let now = Utc::now();
    let diff = now - *when;

    if diff.num_milliseconds() < 0 {
        return 0;
    }

    diff.num_milliseconds().unsigned_abs() as EOInt / SETTINGS.world.tick_rate
}
