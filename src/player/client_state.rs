/// describes the state of a client
#[derive(Debug, Eq, PartialEq, Copy, Clone)]
pub enum ClientState {
    Uninitialized,
    Initialized,
    Accepted,
    LoggedIn,
    EnteringGame,
    InGame,
}
