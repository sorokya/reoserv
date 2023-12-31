use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::MapTileSpec,
        net::{
            server::{ChairPlayerServerPacket, ChairReplyServerPacket, SitState},
            PacketAction, PacketFamily,
        },
        Coords, Direction,
    },
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
            MapTileSpec::ChairDown
                if character.coords.y - 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Down
            }
            MapTileSpec::ChairUp
                if character.coords.y + 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Up
            }
            MapTileSpec::ChairRight
                if character.coords.y == coords.y && character.coords.x - 1 == coords.x =>
            {
                Direction::Right
            }
            MapTileSpec::ChairLeft
                if character.coords.y == coords.y && character.coords.x + 1 == coords.x =>
            {
                Direction::Left
            }
            MapTileSpec::ChairUpLeft
                if character.coords.y + 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Up
            }
            MapTileSpec::ChairUpLeft
                if character.coords.y == coords.y && character.coords.x + 1 == coords.x =>
            {
                Direction::Left
            }
            MapTileSpec::ChairDownRight
                if character.coords.y - 1 == coords.y && character.coords.x == coords.x =>
            {
                Direction::Down
            }
            MapTileSpec::ChairDownRight
                if character.coords.y == coords.y && character.coords.x - 1 == coords.x =>
            {
                Direction::Right
            }
            MapTileSpec::ChairAll => {
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

        let reply = ChairReplyServerPacket {
            player_id,
            coords,
            direction: character.direction,
        };

        let mut writer = EoWriter::new();
        reply.serialize(&mut writer);

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Chair,
            writer.to_byte_array(),
        );

        if character.hidden {
            return;
        }

        let reply = ChairPlayerServerPacket {
            player_id,
            coords,
            direction: character.direction,
        };

        self.send_packet_near_player(player_id, PacketAction::Player, PacketFamily::Chair, reply);
    }
}
