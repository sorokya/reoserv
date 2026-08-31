use eolib::rng::Rng;

/// Creates a new [`Rng`] seeded from OS entropy.
pub fn new_seeded_rng() -> Rng {
    let mut seed_bytes = [0u8; 4];
    getrandom::fill(&mut seed_bytes).expect("Failed to get entropy for RNG seed");
    Rng::new(u32::from_ne_bytes(seed_bytes))
}
