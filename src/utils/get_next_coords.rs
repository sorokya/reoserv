use eo::{data::EOChar, protocol::Coords, protocol::Direction};

pub fn get_next_coords(
    coords: &Coords,
    direction: Direction,
    map_width: EOChar,
    map_height: EOChar,
) -> Coords {
    match direction {
        Direction::Down => {
            if coords.y >= map_height {
                *coords
            } else {
                Coords {
                    x: coords.x,
                    y: coords.y + 1,
                }
            }
        }
        Direction::Left => {
            if coords.x == 0 {
                *coords
            } else {
                Coords {
                    x: coords.x - 1,
                    y: coords.y,
                }
            }
        }
        Direction::Up => {
            if coords.y == 0 {
                *coords
            } else {
                Coords {
                    x: coords.x,
                    y: coords.y - 1,
                }
            }
        }
        Direction::Right => {
            if coords.x >= map_width {
                *coords
            } else {
                Coords {
                    x: coords.x + 1,
                    y: coords.y,
                }
            }
        }
    }
}
