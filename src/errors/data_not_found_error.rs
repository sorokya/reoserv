use eo::data::EOShort;

#[derive(Debug)]
pub struct DataNotFoundError {
    pub kind: String,
    pub id: EOShort,
}

impl DataNotFoundError {
    pub fn new(kind: String, id: EOShort) -> Self {
        Self { kind, id }
    }
}

impl std::error::Error for DataNotFoundError {}

impl std::fmt::Display for DataNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "{} not found: {}", self.kind, self.id)
    }
}
