use eo::data::EOShort;

#[derive(Debug)]
pub struct WrongPlayerIdError {
    pub expected: EOShort,
    pub actual: EOShort,
}

impl std::error::Error for WrongPlayerIdError {}

impl std::fmt::Display for WrongPlayerIdError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(
            f,
            "Wrong player id: expected {}, got {}",
            self.expected, self.actual
        )
    }
}
