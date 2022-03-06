use eo::data::EOShort;

#[derive(Debug)]
pub struct WrongAccountError {
    pub expected: EOShort,
    pub actual: EOShort,
}

impl WrongAccountError {
    pub fn new(expected: EOShort, actual: EOShort) -> Self {
        Self { expected, actual }
    }
}

impl std::error::Error for WrongAccountError {}

impl std::fmt::Display for WrongAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrong account id: expected {}, got {}", self.expected, self.actual)
    }
}