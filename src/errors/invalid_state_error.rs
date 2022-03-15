use crate::player::State;

#[derive(Debug)]
pub struct InvalidStateError {
    pub expected: State,
    pub actual: State,
}

impl InvalidStateError {
    pub fn new(expected: State, actual: State) -> Self {
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
