use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::CharacterPlayerServerPacket, PacketAction, PacketFamily},
};

use crate::{character::Character, player::ClientState};

use super::super::Player;

impl Player {
    pub async fn request_character_deletion(&mut self, character_id: i32) -> bool {
        if self.state != ClientState::LoggedIn {
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

        let character = match Character::load(&mut conn, character_id).await {
            Ok(character) => character,
            Err(_) => {
                self.close(format!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    character_id
                ))
                .await;
                return false;
            }
        };

        if character.account_id != self.account_id {
            self.close(format!(
                "Player {} attempted to delete character ({}) belonging to another account: {}",
                self.account_id, character.name, character.account_id
            ))
            .await;
            return false;
        }

        let session_id = self.generate_session_id();
        let reply = CharacterPlayerServerPacket {
            session_id,
            character_id,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            self.close(format!(
                "Error serializing CharacterPlayerServerPacket: {}",
                e
            ))
            .await;
            return false;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Player,
                PacketFamily::Character,
                writer.to_byte_array(),
            )
            .await;

        true
    }
}
