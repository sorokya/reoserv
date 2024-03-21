use crate::map::WeddingState;

use super::super::Map;

impl Map {
    pub fn say_i_do(&mut self, player_id: i32) {
        let wedding = match self.wedding.as_mut() {
            Some(wedding) => wedding,
            None => return,
        };

        if wedding.partner_id == player_id && wedding.state == WeddingState::WaitingForPartner {
            wedding.state = WeddingState::PartnerAgrees;
            self.wedding_ticks = 0;
        }

        if wedding.player_id == player_id && wedding.state == WeddingState::WaitingForPlayer {
            wedding.state = WeddingState::PlayerAgrees;
            self.wedding_ticks = 0;
        }
    }
}
