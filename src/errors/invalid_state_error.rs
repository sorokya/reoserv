use crate::player::ClientState;

#[derive(Debug)]
pub struct InvalidStateError {
    pub expected: ClientState,
    pub actual: ClientState,
}

impl InvalidStateError {
    pub fn new(expected: ClientState, actual: ClientState) -> Self {
        Self { expected, actual }
    }
}

impl std::error::Error for InvalidStateError {}

impl std::fmt::Display for InvalidStateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "expected state {:?}, got state {:?}",
            self.expected, self.actual
        )
    }
}
