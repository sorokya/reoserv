use std::mem;

use eo::{
    data::{EOShort, EOThree, Serializeable, StreamBuilder},
    protocol::{server::effect, PacketAction, PacketFamily},
};

use super::Map;

impl Map {
    pub fn play_effect(&mut self, player_id: EOShort, effect_id: EOThree) {
        let packet = effect::Player {
            player_id,
            effect_id,
        };

        let mut builder = StreamBuilder::with_capacity(mem::size_of::<effect::Player>());
        packet.serialize(&mut builder);
        let buf = builder.get();

        for (id, character) in self.characters.iter() {
            if let Some(player) = character.player.as_ref() {
                if &player_id != id {
                    player.send(PacketAction::Player, PacketFamily::Effect, buf.clone());
                }
            }
        }
    }
}
