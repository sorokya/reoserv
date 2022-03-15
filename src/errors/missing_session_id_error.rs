#[derive(Debug)]
pub struct MissingSessionIdError;

impl std::error::Error for MissingSessionIdError {}

impl std::fmt::Display for MissingSessionIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Player has no session id")
    }
}
