use crate::map::WeddingState;

use super::super::Map;

impl Map {
    pub fn accept_wedding_request(&mut self, player_id: i32) {
        let wedding = match self.wedding.as_mut() {
            Some(wedding) => wedding,
            None => return,
        };

        if wedding.partner_id != player_id || wedding.state != WeddingState::Requested {
            return;
        }

        wedding.state = WeddingState::Accepted;
        self.wedding_ticks = 0;
    }
}
