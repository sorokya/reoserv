use super::super::World;

impl World {
    pub fn show_captcha(&mut self, victim_name: String, experience: i32) {
        let player_id = match self.characters.get(&victim_name) {
            Some(player_id) => player_id,
            None => return,
        };

        let player = match self.players.get(player_id) {
            Some(player) => player,
            None => return,
        };

        player.show_captcha(experience);
    }
}
