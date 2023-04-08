use eo::data::{EOInt, EOShort};
use tokio::sync::oneshot;

use super::World;

impl World {
    pub fn drop_player(
        &mut self,
        player_id: EOShort,
        account_id: EOInt,
        character_name: &str,
        respond_to: oneshot::Sender<()>,
    ) {
        debug!(
            "Dropping player! id: {}, account_id: {}, character_name: {}",
            player_id, account_id, character_name
        );

        self.players.remove(&player_id).unwrap();

        if account_id > 0 {
            self.accounts.retain(|id| *id != account_id);
        }

        if self.characters.contains_key(character_name) {
            self.characters.remove(character_name);
        }

        let _ = respond_to.send(());
    }
}
