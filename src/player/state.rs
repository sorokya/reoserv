use eo::data::EOShort;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    Uninitialized,
    Initialized,
    LoggedIn {
        account_id: EOShort,
    },
    Playing {
        account_id: EOShort,
        character_id: EOShort,
    },
}
