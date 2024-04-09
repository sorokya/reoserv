use std::cmp;

use eolib::protocol::{
    net::{server::BankReplyServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub fn withdraw_gold(&mut self, player_id: i32, npc_index: i32, amount: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let amount = cmp::min(character.gold_bank, amount);
        if amount == 0 {
            return;
        }

        let amount = character.can_hold(1, amount);
        if amount == 0 {
            return;
        }

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Bank {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.gold_bank -= amount;
        character.add_item(1, amount);

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Reply,
                PacketFamily::Bank,
                &BankReplyServerPacket {
                    gold_inventory: character.get_item_amount(1),
                    gold_bank: character.gold_bank,
                },
            );
        }
    }
}
