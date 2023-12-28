#[derive(Debug)]
pub struct DataNotFoundError {
    pub kind: String,
    pub id: i32,
}

impl DataNotFoundError {
    pub fn new(kind: String, id: i32) -> Self {
        Self { kind, id }
    }
}

impl std::error::Error for DataNotFoundError {}

impl std::fmt::Display for DataNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} not found: {}", self.kind, self.id)
    }
}
