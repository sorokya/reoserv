#[derive(Debug)]
pub struct WrongSessionIdError {
    pub expected: i32,
    pub actual: i32,
}

impl WrongSessionIdError {
    pub fn new(expected: i32, actual: i32) -> Self {
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
