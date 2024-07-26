use eolib::protocol::net::{server::TradeCloseServerPacket, PacketAction, PacketFamily};

use super::Player;

impl Player {
    pub async fn cancel_trade(&mut self) {
        let interact_player_id = match self.interact_player_id {
            Some(player_id) => player_id,
            None => return,
        };

        if !self.trading {
            return;
        }

        self.interact_player_id = None;
        self.trading = false;
        self.trade_accepted = false;

        let _ = self
            .bus
            .send(
                PacketAction::Close,
                PacketFamily::Trade,
                TradeCloseServerPacket {
                    partner_player_id: interact_player_id,
                },
            )
            .await;
    }
}
