use eo::data::EOInt;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum State {
    Uninitialized,
    Initialized,
    LoggedIn,
    Playing,
}
