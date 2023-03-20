use eo::protocol::Coords;

pub fn is_occupied(coords: Coords, occupied_tiles: &[Coords]) -> bool {
    occupied_tiles.contains(&coords)
}
