use eolib::{data::{EoWriter, EoSerialize}, protocol::net::{PacketAction, PacketFamily, server::LockerBuyServerPacket}};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn upgrade_locker(&mut self, player_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.bank_level >= SETTINGS.bank.max_upgrades {
            return;
        }

        let cost = SETTINGS.bank.upgrade_base_cost
            + SETTINGS.bank.upgrade_cost_step * character.bank_level;

        if character.get_item_amount(1) < cost {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, cost);
        character.bank_level += 1;

        let buy = LockerBuyServerPacket {
            gold_amount: character.get_item_amount(1),
            locker_upgrades: character.bank_level,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = buy.serialize(&mut writer) {
            error!("Failed to serialize LockerBuyServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Buy,
            PacketFamily::Locker,
            writer.to_byte_array(),
        );
    }
}
