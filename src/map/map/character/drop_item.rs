use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            client::ByteCoords,
            server::{ItemAddServerPacket, ItemDropServerPacket},
            PacketAction, PacketFamily, ThreeItem,
        },
        r#pub::ItemSpecial,
        Coords,
    },
};

use crate::{utils::get_distance, ITEM_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn drop_item(&mut self, target_player_id: i32, item: ThreeItem, coords: ByteCoords) {
        if item.amount <= 0 || item.amount > SETTINGS.limits.max_item {
            return;
        }

        let item_record = match ITEM_DB.items.get(item.id as usize - 1) {
            Some(item) => item,
            None => return,
        };

        if item_record.special == ItemSpecial::Lore {
            return;
        }

        let (amount_to_drop, coords) = {
            let character = match self.characters.get(&target_player_id) {
                Some(character) => character,
                None => return,
            };

            if character.map_id == SETTINGS.jail.map {
                return;
            }

            if character.player.as_ref().unwrap().is_trading().await {
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
            if distance > SETTINGS.world.drop_distance {
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
            let character = self.characters.get_mut(&target_player_id).unwrap();
            character.remove_item(item.id, amount_to_drop);
        }

        let item_index = self.get_next_item_index(1);

        let character = self.characters.get(&target_player_id).unwrap();
        let weight = character.get_weight();

        let reply = ItemDropServerPacket {
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
            weight,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize ItemDropServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Drop,
            PacketFamily::Item,
            writer.to_byte_array(),
        );

        self.items.insert(
            item_index,
            super::super::Item {
                id: item.id,
                amount: amount_to_drop,
                coords,
                owner: target_player_id,
            },
        );

        let reply = ItemAddServerPacket {
            item_id: item.id,
            item_index,
            item_amount: amount_to_drop,
            coords,
        };

        self.send_packet_near_exclude_player(
            &coords,
            target_player_id,
            PacketAction::Add,
            PacketFamily::Item,
            reply,
        );
    }
}
