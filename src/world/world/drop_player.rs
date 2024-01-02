use tokio::sync::oneshot;

use super::World;

impl World {
    pub fn drop_player(
        &mut self,
        player_id: i32,
        ip: String,
        account_id: i32,
        character_name: &str,
        respond_to: oneshot::Sender<()>,
    ) {
        self.connection_log.remove_connection(&ip);

        if !self.players.contains_key(&player_id) {
            let _ = respond_to.send(());
            return;
        }

        self.players.remove(&player_id);

        if account_id > 0 {
            self.accounts.retain(|id| *id != account_id);
        }

        if self.characters.contains_key(character_name) {
            self.characters.remove(character_name);
        }

        let _ = respond_to.send(());
    }
}
