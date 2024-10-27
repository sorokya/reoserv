use chrono::Utc;

use crate::{player::ClientState, SETTINGS};

use super::Player;

impl Player {
    pub async fn tick(&mut self) {
        self.ping_ticks += 1;

        if self.ping_ticks >= SETTINGS.server.ping_rate {
            self.ping().await;
            self.ping_ticks = 0;
        }

        if self.state == ClientState::Uninitialized {
            let time_since_connection = Utc::now() - self.connected_at;
            if time_since_connection.num_seconds() > SETTINGS.server.hangup_delay.into() {
                self.close(format!(
                    "Failed to start handshake in {} seconds",
                    SETTINGS.server.hangup_delay
                ))
                .await;
            }
        }

        self.bus.log.clean_old_entries();
    }
}
