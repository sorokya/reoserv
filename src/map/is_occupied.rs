use eo::protocol::Coords;

pub fn is_occupied(coords: Coords, occupied_tiles: &Vec<Coords>) -> bool {
    // idk if this is ref check or val check
    occupied_tiles.contains(&coords)
}
