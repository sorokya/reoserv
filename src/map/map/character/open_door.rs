use eo::{
    data::EOShort,
    protocol::{server::door, Coords, PacketAction, PacketFamily},
    pubs::EifItemType,
};

use crate::{utils::in_client_range, ITEM_DB};

use super::super::Map;

impl Map {
    pub fn open_door(&mut self, player_id: EOShort, door_coords: Coords) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let door = match self
            .doors
            .iter_mut()
            .find(|door| door.coords == door_coords)
        {
            Some(door) => door,
            None => return,
        };

        if !in_client_range(&character.coords, &door_coords) {
            return;
        }

        // Key 1 just means it's an unlocked door
        if door.key > 1
            && !character.items.iter().any(|item| {
                let item_data = match ITEM_DB.items.get(item.id as usize - 1) {
                    Some(item_data) => item_data,
                    None => return false,
                };

                item_data.r#type == EifItemType::Key && item_data.spec1 as EOShort == door.key
            })
        {
            return;
        }

        door.open = true;

        let packet = door::Open {
            coords: door_coords,
        };

        self.send_packet_near(&door_coords, PacketAction::Open, PacketFamily::Door, packet);
    }
}
