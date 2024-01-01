use eolib::protocol::{Coords, Direction};

pub fn get_next_coords(
    coords: &Coords,
    direction: Direction,
    map_width: i32,
    map_height: i32,
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
        _ => *coords,
    }
}
