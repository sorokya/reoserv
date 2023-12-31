use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::MapTileSpec,
        net::{
            server::{LockerReplyServerPacket, LockerSpecServerPacket},
            Item, PacketAction, PacketFamily, ThreeItem,
        },
        Coords,
    },
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
        if character.bank.len() as i32 >= bank_size {
            let packet = LockerSpecServerPacket {
                locker_max_items: bank_size,
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize LockerSpecServerPacket: {}", e);
                return;
            }

            character.player.as_ref().unwrap().send(
                PacketAction::Spec,
                PacketFamily::Locker,
                writer.to_byte_array(),
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
            Some(tile) => *tile == MapTileSpec::BankVault,
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

        let packet = LockerReplyServerPacket {
            deposited_item: Item {
                id: item.id,
                amount: character.get_item_amount(item.id),
            },
            weight: character.get_weight(),
            locker_items: character
                .bank
                .iter()
                .map(|i| ThreeItem {
                    id: i.id,
                    amount: i.amount,
                })
                .collect(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize LockerReplyServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Locker,
            writer.to_byte_array(),
        );
    }
}
