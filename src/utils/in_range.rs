use crate::SETTINGS;

pub fn in_range(x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    in_range_distance(x1, y1, x2, y2, SETTINGS.world.see_distance as f64)
}

pub fn in_range_distance(x1: f64, y1: f64, x2: f64, y2: f64, distance: f64) -> bool {
    let dx = x1 - x2;
    let dy = y1 - y2;
    let actual_distance = ((dx * dx) + (dy * dy)).sqrt();
    actual_distance.abs() <= distance
}
