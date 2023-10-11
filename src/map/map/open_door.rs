use eo::{
    data::EOShort,
    protocol::{server::door, Coords, PacketAction, PacketFamily},
};

use crate::utils::in_client_range;

use super::Map;

impl Map {
    pub fn open_door(&self, player_id: EOShort, door_coords: Coords) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if in_client_range(&character.coords, &door_coords) {
            let packet = door::Open {
                coords: door_coords,
            };

            self.send_packet_near(&door_coords, PacketAction::Open, PacketFamily::Door, packet);
        }
    }
}
