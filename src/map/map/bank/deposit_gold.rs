use std::cmp;

use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{server::BankReplyServerPacket, PacketAction, PacketFamily},
};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub async fn deposit_gold(&mut self, player_id: i32, session_id: i32, amount: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        if player.is_trading().await {
            return;
        }

        let actual_session_id = match player.get_session_id().await {
            Ok(session_id) => session_id,
            Err(_) => return,
        };

        if session_id != actual_session_id {
            return;
        }

        let amount = cmp::min(character.get_item_amount(1), amount);
        if amount == 0 {
            return;
        }

        let amount = cmp::min(SETTINGS.limits.max_bank_gold - character.gold_bank, amount);
        if amount == 0 {
            return;
        }

        let interact_npc_index = match player.get_interact_npc_index().await {
            Some(index) => index,
            None => return,
        };

        if !self.npcs.contains_key(&interact_npc_index) {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, amount);
        character.gold_bank += amount;

        let reply = BankReplyServerPacket {
            gold_inventory: character.get_item_amount(1),
            gold_bank: character.gold_bank,
        };

        let mut writer = EoWriter::new();
        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize BankReplyServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Bank,
            writer.to_byte_array(),
        );
    }
}
