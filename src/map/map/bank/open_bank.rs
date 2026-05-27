use eolib::protocol::{
    net::{PacketAction, PacketFamily, server::BankOpenServerPacket},
    r#pub::NpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub fn open_bank(&mut self, player_id: i32, npc_index: i32, session_id: i32) {
        let npc = match self.npcs.iter().find(|npc| npc.index == npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_db = NPC_DB.load();
        let npc_data = match npc_db.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Bank {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        player.set_interact_npc_index(npc_index);

        player.send(
            PacketAction::Open,
            PacketFamily::Bank,
            &BankOpenServerPacket {
                gold_bank: character.gold_bank,
                session_id,
                locker_upgrades: character.bank_level,
            },
        );
    }
}
