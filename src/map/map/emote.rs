use eo::{data::{EOShort, Serializeable, StreamBuilder}, protocol::{Emote, server::emote, PacketAction, PacketFamily}};

use super::Map;

impl Map {
    pub fn emote(&self, target_player_id: EOShort, emote: Emote) {
        if let Some(target) = self.characters.get(&target_player_id) {
            let packet = emote::Player {
                player_id: target_player_id,
                emote,
            };
            let mut builder = StreamBuilder::new();
            packet.serialize(&mut builder);
            let buf = builder.get();
            for character in self.characters.values() {
                if character.player_id.unwrap() != target_player_id
                    && character.is_in_range(target.coords)
                {
                    debug!("Send: {:?}", packet);
                    character.player.as_ref().unwrap().send(
                        PacketAction::Player,
                        PacketFamily::Emote,
                        buf.clone(),
                    );
                }
            }
        }
    }
}