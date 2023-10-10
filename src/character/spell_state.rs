use chrono::{DateTime, Utc};
use eo::data::{EOShort, EOThree};

#[derive(Clone, Debug)]
pub enum SpellState {
    None,
    Requested {
        spell_id: EOShort,
        timestamp: EOThree,
        cast_time: DateTime<Utc>,
    },
}

impl Default for SpellState {
    fn default() -> Self {
        Self::None
    }
}
