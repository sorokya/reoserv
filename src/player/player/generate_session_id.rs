use eolib::data::SHORT_MAX;
use rand::Rng;

use super::Player;

impl Player {
    pub fn generate_session_id(&mut self) -> i32 {
        let mut rng = rand::thread_rng();
        let id = rng.gen_range(1..SHORT_MAX) as i32;
        self.session_id = Some(id);
        id
    }
}
