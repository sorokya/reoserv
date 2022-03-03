#[derive(Debug)]
pub enum State {
    Uninitialized,
    Initialized,
    LoggedIn,
    Playing,
}
