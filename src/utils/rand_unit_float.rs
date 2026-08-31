use eolib::rng::Rng;

/// Generates a pseudo-random `f64` in the range `[0.0, 1.0)`.
pub fn rand_unit_float(rng: &mut Rng) -> f64 {
    rng.rand() as f64 / 2147483648.0
}
