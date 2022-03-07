use eo::data::EOInt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    Uninitialized,
    Initialized,
    LoggedIn {
        account_id: EOInt,
    },
    EnteringWorld {
        account_id: EOInt,
        character_id: EOInt,
    },
    Playing {
        account_id: EOInt,
        character_id: EOInt,
    },
}
