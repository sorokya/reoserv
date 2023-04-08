use eo::{data::EOChar, protocol::Coords};

pub fn get_distance(a: &Coords, b: &Coords) -> EOChar {
    let dx = (a.x as f64 - b.x as f64).abs();
    let dy = (a.y as f64 - b.y as f64).abs();

    (dx + dy) as EOChar
}

pub fn in_range(a: &Coords, b: &Coords) -> bool {
    let distance = get_distance(a, b);

    // TODO: move hard coded values to config
    if a.x > b.x || a.y > b.y {
        distance <= 12
    } else {
        distance <= 15
    }
}
