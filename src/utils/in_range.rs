use crate::SETTINGS;

pub fn in_range(x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    in_range_distance(x1, y1, x2, y2, SETTINGS.world.see_distance as f64)
}

pub fn in_range_distance(x1: f64, y1: f64, x2: f64, y2: f64, _distance: f64) -> bool {
    let dx = (x1 - x2).abs();
    let dy = (y1 - y2).abs();

    if x1 > x2 || y1 > y2 {
        dx + dy <= 12.0
    } else {
        dx + dy <= 15.0
    }
}
