use eolib::protocol::{
    net::{server::DoorOpenServerPacket, PacketAction, PacketFamily},
    r#pub::ItemType,
    Coords,
};

use crate::{utils::in_client_range, ITEM_DB};

use super::super::Map;

impl Map {
    pub fn open_door(&mut self, player_id: i32, coords: Coords) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let door = match self.doors.iter_mut().find(|door| door.coords == coords) {
            Some(door) => door,
            None => return,
        };

        if door.open || !in_client_range(&character.coords, &coords) {
            return;
        }

        // Key 1 just means it's an unlocked door
        if door.key > 1
            && !character.items.iter().any(|item| {
                let item_data = match ITEM_DB.items.get(item.id as usize - 1) {
                    Some(item_data) => item_data,
                    None => return false,
                };

                item_data.r#type == ItemType::Key && item_data.spec1 == door.key
            })
        {
            return;
        }

        door.open = true;

        self.send_packet_near(
            &coords,
            PacketAction::Open,
            PacketFamily::Door,
            DoorOpenServerPacket { coords },
        );
    }
}
