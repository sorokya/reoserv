use eo::data::EOShort;
use tokio::sync::oneshot;

use crate::character::Character;

use super::Map;

impl Map {
    pub fn get_character(&self, player_id: EOShort, respond_to: oneshot::Sender<Option<Box<Character>>>) {
        if let Some(character) = self.characters.get(&player_id) {
            let _ = respond_to.send(Some(Box::new(character.to_owned())));
        } else {
            let _ = respond_to.send(None);
        }
    }
}
