pub fn in_range(x1: f64, y1: f64, x2: f64, y2: f64) -> bool {
    let dx = x1 - x2;
    let dy = y1 - y2;
    let distance = ((dx * dx) + (dy * dy)).sqrt();
    distance <= 11.0 // TODO: make configurable
}
