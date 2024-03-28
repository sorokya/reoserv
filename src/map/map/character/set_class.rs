use super::super::Map;

impl Map {
    pub fn set_class(&mut self, player_id: i32, class_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.class = class_id;
        character.calculate_stats();
    }
}
