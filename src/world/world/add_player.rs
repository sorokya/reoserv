use eo::data::i32;
use tokio::sync::oneshot;

use crate::player::PlayerHandle;

use super::World;

impl World {
    pub fn add_player(
        &mut self,
        player_id: i32,
        player: PlayerHandle,
        respond_to: oneshot::Sender<()>,
    ) {
        self.players.insert(player_id, player);
        let _ = respond_to.send(());
    }
}
