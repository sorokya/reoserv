use eo::data::i32;

use super::World;

impl World {
    pub fn get_next_player_id(&self, seed: i32) -> i32 {
        if self.players.iter().any(|(id, _)| *id == seed) {
            self.get_next_player_id(seed + 1)
        } else {
            seed
        }
    }
}
