use std::cmp;

use eo::{
    data::{i32, EOInt, i32, StreamBuilder},
    protocol::{Coords, Item, PacketAction, PacketFamily},
    pubs::EmfTileSpec,
};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub async fn add_locker_item(&mut self, player_id: i32, item: Item) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.player.as_ref().unwrap().is_trading().await {
            return;
        }

        let bank_size = SETTINGS.bank.base_size + character.bank_level * SETTINGS.bank.size_step;
        if character.bank.len() as EOInt >= bank_size {
            let mut builder = StreamBuilder::new();
            builder.add_char(bank_size as i32);
            character.player.as_ref().unwrap().send(
                PacketAction::Spec,
                PacketFamily::Locker,
                builder.get(),
            );
            return;
        }

        let adjacent_tiles = [
            self.get_tile(&Coords {
                x: character.coords.x,
                y: character.coords.y - 1,
            }),
            self.get_tile(&Coords {
                x: character.coords.x,
                y: character.coords.y + 1,
            }),
            self.get_tile(&Coords {
                x: character.coords.x - 1,
                y: character.coords.y,
            }),
            self.get_tile(&Coords {
                x: character.coords.x + 1,
                y: character.coords.y,
            }),
        ];

        if !adjacent_tiles.iter().any(|tile| match tile {
            Some(tile) => *tile == EmfTileSpec::BankVault,
            None => false,
        }) {
            return;
        }

        let amount = cmp::min(character.get_item_amount(item.id), item.amount);
        if amount == 0 {
            return;
        }

        let amount = cmp::min(character.can_bank_hold(item.id, amount), amount);
        if amount == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(item.id, amount);
        character.add_bank_item(item.id, amount);

        let mut builder = StreamBuilder::new();
        builder.add_short(item.id);
        builder.add_int(character.get_item_amount(item.id));

        let weight = character.get_weight();
        builder.add_char(weight.current);
        builder.add_char(weight.max);

        for item in &character.bank {
            builder.add_short(item.id);
            builder.add_three(item.amount);
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Locker,
            builder.get(),
        );
    }
}
