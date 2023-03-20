use eo::{
    data::Serializeable,
    protocol::{server::talk, PacketAction, PacketFamily},
};

use super::super::World;

impl World {
    pub async fn broadcast_announcement(&self, name: &str, message: &str) {
        let packet = talk::Announce {
            player_name: name.to_string(),
            message: message.to_string(),
        };
        let buf = packet.serialize();
        for player in self.players.values() {
            if let Ok(character) = player.get_character().await {
                if character.name != name {
                    player.send(PacketAction::Announce, PacketFamily::Talk, buf.clone());
                }
            }
        }
    }
}
