use eo::{
    data::{i32, Serializeable, StreamBuilder},
    protocol::{
        server::chair::{Player, Reply},
        Coords, Direction, PacketAction, PacketFamily, SitState,
    },
    pubs::EmfTileSpec,
};

use crate::utils::get_distance;

use super::super::Map;

impl Map {
    pub fn sit_chair(&mut self, player_id: i32, coords: Coords) {
        if self.is_tile_occupied(&coords) {
            return;
        }

        let tile = match self.get_tile(&coords) {
            Some(tile) => tile,
            None => return,
        };

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

        if get_distance(&character.coords, &coords) > 1 {
            return;
        }

        character.direction = match tile {
            EmfTileSpec::ChairDown
                if character.coords.y - 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Down
            }
            EmfTileSpec::ChairUp
                if character.coords.y + 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Up
            }
            EmfTileSpec::ChairRight
                if character.coords.y == coords.y && character.coords.x - 1 == coords.x =>
            {
                Direction::Right
            }
            EmfTileSpec::ChairLeft
                if character.coords.y == coords.y && character.coords.x + 1 == coords.x =>
            {
                Direction::Left
            }
            EmfTileSpec::ChairUpLeft
                if character.coords.y + 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Up
            }
            EmfTileSpec::ChairUpLeft
                if character.coords.y == coords.y && character.coords.x + 1 == coords.x =>
            {
                Direction::Left
            }
            EmfTileSpec::ChairDownRight
                if character.coords.y - 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Down
            }
            EmfTileSpec::ChairDownRight
                if character.coords.y == coords.y && character.coords.x - 1 == coords.x =>
            {
                Direction::Right
            }
            EmfTileSpec::ChairAll => {
                if character.coords.y - 1 == coords.y && character.coords.x == coords.x {
                    Direction::Down
                } else if character.coords.y + 1 == coords.y && character.coords.x == coords.x {
                    Direction::Up
                } else if character.coords.y == coords.y && character.coords.x - 1 == coords.x {
                    Direction::Right
                } else if character.coords.y == coords.y && character.coords.x + 1 == coords.x {
                    Direction::Left
                } else {
                    return;
                }
            }
            _ => return,
        };

        character.coords = coords;
        character.sit_state = SitState::Chair;

        let reply = Reply {
            player_id,
            coords,
            direction: character.direction,
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Chair,
            builder.get(),
        );

        if character.hidden {
            return;
        }

        let reply = Player {
            player_id,
            coords,
            direction: character.direction,
        };

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Chair, reply);
    }
}
