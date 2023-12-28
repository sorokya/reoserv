use tokio::sync::oneshot;

use crate::character::Character;

use super::super::Map;

impl Map {
    pub fn get_character(
        &self,
        player_id: i32,
        respond_to: oneshot::Sender<Option<Box<Character>>>,
    ) {
        if let Some(character) = self.characters.get(&player_id) {
            let _ = respond_to.send(Some(Box::new(character.to_owned())));
        } else {
            let _ = respond_to.send(None);
        }
    }
}
