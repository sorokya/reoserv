use eo::{
    data::EOShort,
    world::{Coords, Direction},
};

use crate::SETTINGS;

pub fn get_new_viewable_coords(
    target_coords: Coords,
    direction: Direction,
    max_width: EOShort,
    max_height: EOShort,
) -> Vec<Coords> {
    let mut new_coords = Vec::new();
    let see_distance = SETTINGS.world.see_distance as i32;
    for i in -see_distance..see_distance {
        match direction {
            Direction::Up => {
                let x = target_coords.x as i32 + i;
                let y = target_coords.y as i32 - see_distance - i.abs();
                if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                    new_coords.push(Coords::new(x as EOShort, y as EOShort));
                }
            }
            Direction::Down => {
                let x = target_coords.x as i32 + i;
                let y = target_coords.y as i32 + see_distance + i.abs();
                if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                    new_coords.push(Coords {
                        x: x as u16,
                        y: y as u16,
                    });
                }
            }
            Direction::Left => {
                let x = target_coords.x as i32 - see_distance - i.abs();
                let y = target_coords.y as i32 + i;
                if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                    new_coords.push(Coords {
                        x: x as u16,
                        y: y as u16,
                    });
                }
            }
            Direction::Right => {
                let x = target_coords.x as i32 + see_distance + i.abs();
                let y = target_coords.y as i32 + i;
                if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                    new_coords.push(Coords {
                        x: x as u16,
                        y: y as u16,
                    });
                }
            }
        }
    }
    new_coords
}
