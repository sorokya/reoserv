use eo::data::EOShort;

#[derive(Debug)]
pub struct MapNotFoundError {
    pub map_id: EOShort,
}

impl MapNotFoundError {
    pub fn new(map_id: EOShort) -> Self {
        Self { map_id }
    }
}

impl std::error::Error for MapNotFoundError {}

impl std::fmt::Display for MapNotFoundError {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        write!(f, "Map not found: {}", self.map_id)
    }
}
