use eolib::data::SHORT_MAX;

use super::Player;

impl Player {
    pub fn generate_session_id(&mut self) -> i32 {
        let id = self.rng.rand_range(1..SHORT_MAX as u32 - 1) as i32;
        self.session_id = Some(id);
        id
    }
}
