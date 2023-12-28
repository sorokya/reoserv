use eo::{
    data::{i32, Serializeable, StreamBuilder},
    protocol::{
        server::{chair, sit},
        PacketAction, PacketFamily, SitState,
    },
};

use crate::utils::get_next_coords;

use super::super::Map;

impl Map {
    pub fn stand(&mut self, player_id: i32) {
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

                if !character.hidden {
                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Remove,
                        PacketFamily::Sit,
                        reply,
                    );
                }
            }
            SitState::Chair => {
                character.sit_state = SitState::Stand;

                character.coords = get_next_coords(
                    &character.coords,
                    character.direction,
                    self.file.width,
                    self.file.height,
                );

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

                if !character.hidden {
                    self.send_packet_near_player(
                        player_id,
                        PacketAction::Remove,
                        PacketFamily::Sit,
                        reply,
                    );
                }
            }
            _ => {}
        }
    }
}
