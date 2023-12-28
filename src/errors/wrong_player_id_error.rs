use eo::data::i32;

#[derive(Debug)]
pub struct WrongPlayerIdError {
    pub expected: i32,
    pub actual: i32,
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
