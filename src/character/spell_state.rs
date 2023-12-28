use chrono::{DateTime, Utc};
use eo::data::{i32, EOThree};

#[derive(Clone, Debug)]
pub enum SpellState {
    None,
    Requested {
        spell_id: i32,
        timestamp: EOThree,
        cast_time: DateTime<Utc>,
    },
}

impl Default for SpellState {
    fn default() -> Self {
        Self::None
    }
}
