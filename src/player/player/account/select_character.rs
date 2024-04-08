
use eolib::protocol::net::server::{
    WelcomeCode, WelcomeReplyServerPacket, WelcomeReplyServerPacketWelcomeCodeData,
};
use eolib::protocol::net::{PacketAction, PacketFamily};
use eolib::protocol::Coords;

use crate::character::Character;
use crate::errors::DataNotFoundError;
use crate::player::{ClientState, PlayerHandle};
use crate::SETTINGS;

use super::super::Player;

impl Player {
    pub async fn select_character(
        &mut self,
        player_handle: PlayerHandle,
        character_id: i32,
    ) -> bool {
        if self.state != ClientState::LoggedIn {
            return true;
        }

        let player_count = self.world.get_player_count().await;
        if player_count >= SETTINGS.server.max_players {
            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Welcome,
                    WelcomeReplyServerPacket {
                        welcome_code: WelcomeCode::ServerBusy,
                        welcome_code_data: None,
                    },
                )
                .await;

            return true;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                self.close(format!("Error getting connection from pool: {}", e))
                    .await;
                return false;
            }
        };

        let mut character = match Character::load(&mut conn, character_id).await {
            Ok(character) => character,
            Err(e) => {
                self.close(format!("Failed to load character {}: {}", character_id, e))
                    .await;
                return false;
            }
        };

        if character.account_id != self.account_id {
            self.close(format!(
                "Player {} attempted to login to character ({}) belonging to another account: {}",
                self.account_id, character.name, character.account_id
            ))
            .await;
            return false;
        }

        character.player_id = Some(self.id);
        character.player = Some(player_handle);
        character.logged_in_at = Some(chrono::Utc::now());

        character.calculate_stats();

        if self.world.get_map(character.map_id).await.is_err() {
            if self.world.get_map(SETTINGS.rescue.map).await.is_ok() {
                character.map_id = SETTINGS.rescue.map;
                character.coords = Coords {
                    x: SETTINGS.rescue.x,
                    y: SETTINGS.rescue.y,
                };
            } else {
                self.close(format!(
                    "Rescue map not found! {}",
                    DataNotFoundError::new("map".to_string(), SETTINGS.rescue.map,)
                ))
                .await;
                return false;
            }
        }

        let select_character = match self.get_welcome_request_data(&character).await {
            Ok(select_character) => select_character,
            Err(e) => {
                self.close(format!("Error getting welcome request data: {}", e))
                    .await;
                return false;
            }
        };

        self.world
            .add_character(self.id, character.name.clone(), character.guild_tag.clone());

        self.character = Some(character);
        self.state = ClientState::EnteringGame;

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Welcome,
                WelcomeReplyServerPacket {
                    welcome_code: WelcomeCode::SelectCharacter,
                    welcome_code_data: Some(
                        WelcomeReplyServerPacketWelcomeCodeData::SelectCharacter(select_character),
                    ),
                },
            )
            .await;

        true
    }
}
