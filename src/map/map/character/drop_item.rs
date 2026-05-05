use std::cmp;

use eolib::protocol::{
    Coords,
    net::{
        PacketAction, PacketFamily, ThreeItem,
        client::ByteCoords,
        server::{ItemAddServerPacket, ItemDropServerPacket},
    },
    r#pub::ItemSpecial,
};

use crate::{ITEM_DB, SETTINGS, utils::get_distance};

use super::super::Map;

impl Map {
    pub fn drop_item(&mut self, player_id: i32, item: ThreeItem, coords: ByteCoords) {
        if item.amount <= 0
            || item.amount > SETTINGS.load().limits.max_item
            || SETTINGS.load().items.protected_items.contains(&item.id)
        {
            return;
        }

        let item_db = ITEM_DB.load();
        let item_record = match item_db.items.get(item.id as usize - 1) {
            Some(item) => item,
            None => return,
        };

        if item_record.special == ItemSpecial::Lore {
            return;
        }

        let (amount_to_drop, coords) = {
            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => return,
            };

            if character.map_id == SETTINGS.load().jail.map {
                return;
            }

            let coords = match coords {
                ByteCoords { x: 0xFF, y: 0xFF } => character.coords,
                coords => Coords {
                    x: coords.x as i32 - 1,
                    y: coords.y as i32 - 1,
                },
            };

            let distance = get_distance(&coords, &character.coords);
            if distance > SETTINGS.load().world.drop_distance {
                return;
            }

            let actual_item = match character.items.iter().find(|i| i.id == item.id) {
                Some(item) => item,
                None => return,
            };

            let amount_to_drop = cmp::min(item.amount, actual_item.amount);

            if !self.is_tile_walkable(&coords) {
                return;
            }

            (amount_to_drop, coords)
        };

        {
            let character = match self.characters.get_mut(&player_id) {
                Some(character) => character,
                None => return,
            };

            character.remove_item(item.id, amount_to_drop);
        }

        let item_index = match self.add_item(
            item.id,
            amount_to_drop,
            coords,
            player_id,
            SETTINGS.load().world.drop_protect_player,
        ) {
            Ok(index) => index,
            Err(e) => {
                tracing::error!("Failed to add dropped item to map: {}", e);
                return;
            }
        };

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Drop,
                PacketFamily::Item,
                &ItemDropServerPacket {
                    dropped_item: ThreeItem {
                        id: item.id,
                        amount: amount_to_drop,
                    },
                    item_index,
                    remaining_amount: match character.items.iter().find(|i| i.id == item.id) {
                        Some(item) => item.amount,
                        None => 0,
                    },
                    coords,
                    weight: character.get_weight(),
                },
            );
        }

        self.send_packet_near_exclude_player(
            &coords,
            player_id,
            PacketAction::Add,
            PacketFamily::Item,
            &ItemAddServerPacket {
                item_id: item.id,
                item_index,
                item_amount: amount_to_drop,
                coords,
            },
        );
    }
}
