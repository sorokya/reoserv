use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::{chair, sit},
        Direction, PacketAction, PacketFamily, SitState,
    },
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

        match character.sit_state {
            SitState::Floor => {
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

                self.send_packet_near_player(
                    player_id,
                    PacketAction::Remove,
                    PacketFamily::Sit,
                    reply,
                );
            }
            SitState::Chair => {
                character.sit_state = SitState::Stand;

                match character.direction {
                    Direction::Down => character.coords.y += 1,
                    Direction::Left => character.coords.x -= 1,
                    Direction::Up => character.coords.y -= 1,
                    Direction::Right => character.coords.x += 1,
                }

                let reply = chair::Close {
                    player_id,
                    coords: character.coords,
                };

                let mut builder = StreamBuilder::new();
                reply.serialize(&mut builder);

                character.player.as_ref().unwrap().send(
                    PacketAction::Close,
                    PacketFamily::Chair,
                    builder.get(),
                );

                self.send_packet_near_player(
                    player_id,
                    PacketAction::Remove,
                    PacketFamily::Sit,
                    reply,
                );
            }
            _ => {}
        }
    }
}
