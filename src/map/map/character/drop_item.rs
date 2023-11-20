use std::cmp;

use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::item, Coords, PacketAction, PacketFamily, ShortItem},
    pubs::EifItemSpecial,
};

use crate::{utils::get_distance, ITEM_DB, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn drop_item(&mut self, target_player_id: EOShort, item: ShortItem, coords: Coords) {
        if item.amount == 0 {
            return;
        }

        let item_record = match ITEM_DB.items.get(item.id as usize - 1) {
            Some(item) => item,
            None => return,
        };

        if item_record.special == EifItemSpecial::Lore {
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
                Coords { x: 0xFE, y: 0xFE } => character.coords,
                coords => coords,
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

        let reply = item::Drop {
            item_id: item.id,
            amount_dropped: amount_to_drop,
            item_index,
            amount_remaining: match character.items.iter().find(|i| i.id == item.id) {
                Some(item) => item.amount,
                None => 0,
            },
            coords,
            weight,
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        let buf = builder.get();
        character
            .player
            .as_ref()
            .unwrap()
            .send(PacketAction::Drop, PacketFamily::Item, buf);

        self.items.insert(
            item_index,
            super::super::Item {
                id: item.id,
                amount: amount_to_drop,
                coords,
                owner: target_player_id,
            },
        );

        let reply = item::Add {
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
