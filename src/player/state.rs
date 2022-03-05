use eo::data::{EOShort, EOChar};

#[derive(Debug)]
pub enum State {
    Uninitialized,
    Initialized,
    LoggedIn {
        account_id: EOShort,
        num_of_characters: EOChar,
    },
    Playing,
}
