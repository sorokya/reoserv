use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::sit, PacketAction, PacketFamily, SitState},
};

use super::Map;

impl Map {
    pub fn stand(&mut self, player_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character {}", player_id);
                return;
            }
        };

        if character.sit_state == SitState::Stand {
            return;
        }

        character.sit_state = SitState::Stand;

        let reply = sit::Close {
            player_id,
            coords: character.coords,
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Close,
            PacketFamily::Sit,
            builder.get(),
        );

        self.send_packet_near_player(player_id, PacketAction::Remove, PacketFamily::Sit, reply);
    }
}
