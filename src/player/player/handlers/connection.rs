use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::ConnectionAcceptClientPacket, PacketAction},
};

use crate::player::ClientState;

use super::super::Player;

impl Player {
    async fn connection_accept(&mut self, reader: EoReader) {
        let accept = match ConnectionAcceptClientPacket::deserialize(&reader) {
            Ok(accept) => accept,
            Err(e) => {
                self.close(format!(
                    "Failed to deserialize ConnectionAcceptClientPacket: {}",
                    e
                ))
                .await;
                return;
            }
        };

        if accept.player_id != self.id {
            self.close(format!(
                "sending invalid connection id: Got {}, expected {}.",
                accept.player_id, self.id
            ))
            .await;
            return;
        }

        if self.bus.client_enryption_multiple as i32 != accept.client_encryption_multiple
            || self.bus.server_enryption_multiple as i32 != accept.server_encryption_multiple
        {
            self.close(format!(
            "sending invalid encoding multiples: Got server: {}, client: {}. Expected server: {}, client: {}.",
            accept.server_encryption_multiple, accept.client_encryption_multiple, self.bus.server_enryption_multiple, self.bus.client_enryption_multiple
        )).await;
            return;
        }

        self.state = ClientState::Accepted;
    }

    pub async fn handle_connection(&mut self, action: PacketAction, reader: EoReader) {
        match action {
            PacketAction::Accept => self.connection_accept(reader).await,
            PacketAction::Ping => self.bus.need_pong = false,
            _ => error!("Unhandled packet Connection_{:?}", action),
        }
    }
}
