use eolib::{protocol::net::{server::CharacterPlayerServerPacket, PacketAction, PacketFamily}, data::{EoWriter, EoSerialize}};

use crate::character::Character;

use super::super::World;

impl World {
    pub async fn request_character_deletion(&self, player_id: i32, character_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let account_id = match player.get_account_id().await {
            Ok(account_id) => account_id,
            Err(e) => {
                player.close(format!("Error getting account id: {}", e));
                return;
            }
        };

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                player.close(format!("Error getting connection from pool: {}", e));
                return;
            }
        };

        let character = match Character::load(&mut conn, character_id).await {
            Ok(character) => character,
            Err(_) => {
                player.close(format!(
                    "Tried to request character deletion for a character that doesn't exist: {}",
                    character_id
                ));
                return;
            }
        };

        if character.account_id != account_id {
            player.close(format!(
                "Player {} attempted to delete character ({}) belonging to another account: {}",
                account_id, character.name, character.account_id
            ));
            return;
        }

        let session_id = match player.generate_session_id().await {
            Ok(session_id) => session_id,
            Err(e) => {
                player.close(format!("Error generating session id: {}", e));
                return;
            }
        };

        let reply = CharacterPlayerServerPacket {
            session_id,
            character_id,
        };

        let mut writer = EoWriter::new();
        reply.serialize(&mut writer);
        player.send(PacketAction::Player, PacketFamily::Character, writer.to_byte_array());
    }
}
