use eo::protocol::Coords;

use super::super::Map;

impl Map {
    pub fn is_in_bounds(&self, coords: Coords) -> bool {
        coords.x <= self.file.width && coords.y <= self.file.height
    }
}
