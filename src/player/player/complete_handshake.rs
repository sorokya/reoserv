use crate::player::ClientState;

use super::Player;

impl Player {
    pub async fn complete_handshake(
        &mut self,
        player_id: i32,
        client_encryption_multiple: i32,
        server_encryption_multiple: i32,
    ) -> bool {
        if player_id != self.id {
            self.close(format!(
                "sending invalid connection id: Got {}, expected {}.",
                player_id, self.id
            ))
            .await;
            return false;
        }

        if self.bus.client_enryption_multiple as i32 != client_encryption_multiple
            || self.bus.server_enryption_multiple as i32 != server_encryption_multiple
        {
            self.close(format!(
            "sending invalid encoding multiples: Got server: {}, client: {}. Expected server: {}, client: {}.",
            server_encryption_multiple, client_encryption_multiple, self.bus.server_enryption_multiple, self.bus.client_enryption_multiple
        )).await;
            return false;
        }

        self.state = ClientState::Accepted;

        true
    }
}
