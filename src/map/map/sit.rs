use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::sit, PacketAction, PacketFamily, SitState},
};

use super::Map;

impl Map {
    pub fn sit(&mut self, player_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => {
                error!("Failed to get character {}", player_id);
                return;
            }
        };

        if character.sit_state != SitState::Stand {
            return;
        }

        character.sit_state = SitState::Floor;

        let reply = sit::Reply {
            player_id,
            coords: character.coords,
            direction: character.direction,
        };

        debug!("{:?}", reply);

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Sit,
            builder.get(),
        );

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Sit, reply);
    }
}
