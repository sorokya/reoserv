use eo::data::EOShort;

#[derive(Debug)]
pub struct WrongSessionIdError {
    pub expected: EOShort,
    pub actual: EOShort,
}

impl WrongSessionIdError {
    pub fn new(expected: EOShort, actual: EOShort) -> Self {
        Self { expected, actual }
    }
}

impl std::error::Error for WrongSessionIdError {}

impl std::fmt::Display for WrongSessionIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Wrong session id: expected {}, got {}",
            self.expected, self.actual
        )
    }
}
