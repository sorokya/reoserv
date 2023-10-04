use eo::{data::EOChar, protocol::Coords};

pub fn get_distance(a: &Coords, b: &Coords) -> EOChar {
    let dx = (a.x as f64 - b.x as f64).abs();
    let dy = (a.y as f64 - b.y as f64).abs();

    (dx + dy) as EOChar
}

pub fn in_range(observer: &Coords, other: &Coords) -> bool {
    let distance = get_distance(observer, other);

    if observer.x >= other.x || observer.y >= other.y {
        distance <= 12
    } else {
        distance <= 15
    }
}

pub fn in_client_range(observer: &Coords, other: &Coords) -> bool {
    let distance = get_distance(observer, other);

    if observer.x >= other.x || observer.y >= other.y {
        distance <= 11
    } else {
        distance <= 14
    }
}
