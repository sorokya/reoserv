use eo::data::EOShort;
use tokio::sync::oneshot;

use crate::player::PlayerHandle;

use super::World;

impl World {
    pub fn add_player(
        &mut self,
        player_id: EOShort,
        player: PlayerHandle,
        respond_to: oneshot::Sender<()>,
    ) {
        self.players.insert(player_id, player);
        let _ = respond_to.send(());
    }
}
