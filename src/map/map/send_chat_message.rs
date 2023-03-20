use eo::{data::{EOShort, Serializeable}, protocol::{server::talk, PacketAction, PacketFamily}};

use super::Map;

impl Map {
    pub fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        if let Some(target) = self.characters.get(&target_player_id) {
            let packet = talk::Player {
                player_id: target_player_id,
                message,
            };
            let buf = packet.serialize();
            for character in self.characters.values() {
                if target_player_id != character.player_id.unwrap()
                    && target.is_in_range(character.coords)
                {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Talk,
                        buf.clone(),
                    );
                }
            }
        }
    }
}