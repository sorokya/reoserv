use super::super::Map;

impl Map {
    pub fn open_captcha(&mut self, player_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.is_deep {
            return;
        }

        character.captcha_open = true;
    }
}
