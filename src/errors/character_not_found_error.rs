#[derive(Debug)]
pub struct CharacterNotFoundError {
    pub name: String,
}

impl CharacterNotFoundError {
    pub fn new(name: String) -> Self {
        Self { name }
    }
}

impl std::error::Error for CharacterNotFoundError {}

impl std::fmt::Display for CharacterNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Character not found: {}", self.name)
    }
}
