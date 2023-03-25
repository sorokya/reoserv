use eo::{data::{EOShort, StreamBuilder, Serializeable}, protocol::{server::item, ShortItem, Weight, PacketAction, PacketFamily}};

use crate::{utils::get_distance, SETTINGS};

use super::Map;

impl Map {
    pub fn get_item(&mut self, target_player_id: EOShort, item_index: EOShort) {
        let item = self.items.get_mut(&item_index);

        if item.is_none() {
            return;
        }

        let character = self.characters.get_mut(&target_player_id);

        if character.is_none() {
            return;
        }

        let item = item.unwrap();
        let character = character.unwrap();

        let distance = get_distance(&item.coords, &character.coords);
        if distance > SETTINGS.world.drop_distance {
            return;
        }

        let amount = character.can_hold(item.id, item.amount);
        if amount == 0 {
            return;
        }

        character.add_item(item.id, amount);

        let reply = item::Get {
            taken_item_index: item_index,
            taken_item: ShortItem {
                id: item.id,
                amount,
            },
            weight: Weight {
                current: character.weight,
                max: character.max_weight,
            },
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        let buf = builder.get();

        character.player.as_ref().unwrap().send(
            PacketAction::Get,
            PacketFamily::Item,
            buf,
        );

        if amount == item.amount {
            self.items.remove(&item_index);
            return;
        }

        let reply = item::Remove {
            item_index,
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        let buf = builder.get();

        for character in self.characters.values() {
            if character.is_in_range(&item.coords) {
                character.player.as_ref().unwrap().send(
                    PacketAction::Remove,
                    PacketFamily::Item,
                    buf.clone(),
                );
            }
        }

        if amount != item.amount {
            item.amount -= amount;

            let reply = item::Add {
                item_id: item.id,
                item_index,
                item_amount: item.amount,
                coords: item.coords,
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            let buf = builder.get();

            for character in self.characters.values() {
                if character.is_in_range(&item.coords) {
                    character.player.as_ref().unwrap().send(
                        PacketAction::Add,
                        PacketFamily::Item,
                        buf.clone(),
                    );
                }
            }
        }
    }
}