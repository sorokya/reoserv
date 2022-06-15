use eo::{
    data::{EOShort, EOChar},
    world::{TinyCoords, Direction},
};

use crate::SETTINGS;

pub fn get_new_viewable_coords(
    target_coords: TinyCoords,
    direction: Direction,
    max_width: EOShort,
    max_height: EOShort,
) -> Vec<TinyCoords> {
    let mut new_coords = Vec::new();
    let see_distance = SETTINGS.world.see_distance as i32;
    let edge_size = 5;

    // Calculate newly visible coordinates based on the direction the target moves
    match direction {
        Direction::Up => {
            for y in (target_coords.y as i32 - see_distance)..(target_coords.y as i32 - see_distance + edge_size) {
                for x in (target_coords.x as i32 - see_distance)..(target_coords.x as i32 + see_distance) {
                    if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                        new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
                    }
                }
            }
        }
        Direction::Right => {
            for x in (target_coords.x as i32 + see_distance - edge_size)..(target_coords.x as i32 + see_distance) {
                for y in (target_coords.y as i32 - see_distance)..(target_coords.y as i32 + see_distance) {
                    if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                        new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
                    }
                }
            }
        }
        Direction::Down => {
            for y in (target_coords.y as i32 + see_distance - edge_size)..(target_coords.y as i32 + see_distance) {
                for x in (target_coords.x as i32 - see_distance)..(target_coords.x as i32 + see_distance) {
                    if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                        new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
                    }
                }
            }
        }
        Direction::Left => {
            for x in (target_coords.x as i32 - see_distance)..(target_coords.x as i32 - see_distance + edge_size) {
                for y in (target_coords.y as i32 - see_distance)..(target_coords.y as i32 + see_distance) {
                    if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
                        new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
                    }
                }
            }
        }
    }



    // for i in -see_distance..see_distance {
    //     match direction {
    //         Direction::Up => {
    //             let x = target_coords.x as i32 + i;
    //             let y = target_coords.y as i32 - see_distance - i.abs();
    //             if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
    //                 new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
    //             }
    //         }
    //         Direction::Down => {
    //             let x = target_coords.x as i32 + i;
    //             let y = target_coords.y as i32 + see_distance + i.abs();
    //             if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
    //                 new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
    //             }
    //         }
    //         Direction::Left => {
    //             let x = target_coords.x as i32 - see_distance - i.abs();
    //             let y = target_coords.y as i32 + i;
    //             if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
    //                 new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
    //             }
    //         }
    //         Direction::Right => {
    //             let x = target_coords.x as i32 + see_distance + i.abs();
    //             let y = target_coords.y as i32 + i;
    //             if x >= 0 && x < max_width as i32 && y >= 0 && y < max_height as i32 {
    //                 new_coords.push(TinyCoords::new(x as EOChar, y as EOChar));
    //             }
    //         }
    //     }
    // }

    // draw the new coordinates as a grid
    for y in 0..max_height as EOChar {
        for x in 0..max_width as EOChar {
            if new_coords.contains(&TinyCoords::new(x, y)) {
                print!("#");
            } else {
                print!(" ");
            }
        }
        println!();
    }

    println!();

    for y in 0..max_height as EOChar {
        print!("{}[", y);
        for x in 0..max_width as EOChar {
            if new_coords.contains(&TinyCoords::new(x, y)) {
                print!("{},", x);
            }
        }
        println!("]");
    }


    new_coords
}
