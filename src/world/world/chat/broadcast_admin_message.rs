use eo::{
    data::{EOChar, Serializeable},
    protocol::{server::talk, AdminLevel, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub async fn broadcast_admin_message(
        &self,
        name: &str,
        message: &str,
    ) {
        let packet = talk::Admin {
            player_name: name.to_string(),
            message: message.to_string(),
        };
        let buf = packet.serialize();
        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != name
                    && character.admin_level as EOChar >= AdminLevel::Guardian as EOChar
                {
                    player.send(PacketAction::Admin, PacketFamily::Talk, buf.clone());
                }
            }
        }
    }
}


