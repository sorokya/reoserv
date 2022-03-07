use eo::data::EOInt;

#[derive(Debug)]
pub struct WrongAccountError {
    pub expected: EOInt,
    pub actual: EOInt,
}

impl WrongAccountError {
    pub fn new(expected: EOInt, actual: EOInt) -> Self {
        Self { expected, actual }
    }
}

impl std::error::Error for WrongAccountError {}

impl std::fmt::Display for WrongAccountError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Wrong account id: expected {}, got {}", self.expected, self.actual)
    }
}