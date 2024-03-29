use eolib::protocol::net::{
    server::{ItemAddServerPacket, ItemGetServerPacket, ItemRemoveServerPacket},
    PacketAction, PacketFamily, ThreeItem,
};

use crate::{utils::get_distance, SETTINGS};

use super::super::Map;

impl Map {
    pub fn get_item(&mut self, target_player_id: i32, item_index: i32) {
        let (item_id, item_amount, item_coords) = match self.items.get(&item_index) {
            Some(item) => (item.id, item.amount, item.coords),
            None => return,
        };

        let character = match self.characters.get_mut(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        let distance = get_distance(&item_coords, &character.coords);
        if distance > SETTINGS.world.drop_distance {
            return;
        }

        let amount_picked_up = character.can_hold(item_id, item_amount);
        if amount_picked_up == 0 {
            return;
        }

        character.add_item(item_id, amount_picked_up);

        character.player.as_ref().unwrap().send(
            PacketAction::Get,
            PacketFamily::Item,
            &ItemGetServerPacket {
                taken_item_index: item_index,
                taken_item: ThreeItem {
                    id: item_id,
                    amount: amount_picked_up,
                },
                weight: character.get_weight(),
            },
        );

        if amount_picked_up == item_amount {
            self.items.remove(&item_index);
        } else {
            match self.items.get_mut(&item_index) {
                Some(item) => item.amount -= amount_picked_up,
                None => {
                    error!("Failed to get item {}", item_index);
                    return;
                }
            }
        }

        let reply = ItemRemoveServerPacket { item_index };

        self.send_packet_near(
            &item_coords,
            PacketAction::Remove,
            PacketFamily::Item,
            reply,
        );

        if amount_picked_up != item_amount {
            let reply = ItemAddServerPacket {
                item_id,
                item_index,
                item_amount: item_amount - amount_picked_up,
                coords: item_coords,
            };

            self.send_packet_near(&item_coords, PacketAction::Add, PacketFamily::Item, reply);
        }
    }
}
