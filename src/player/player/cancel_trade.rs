use eo::{
    data::{EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use super::Player;

impl Player {
    pub async fn cancel_trade(&mut self, player_id: EOShort) {
        let interact_player_id = match self.interact_player_id {
            Some(player_id) => player_id,
            None => return,
        };

        if interact_player_id != player_id {
            return;
        }

        self.interact_player_id = None;
        self.trading = false;

        let mut builder = StreamBuilder::new();
        builder.add_short(interact_player_id);
        let _ = self
            .bus
            .send(PacketAction::Close, PacketFamily::Trade, builder.get())
            .await;
    }
}
