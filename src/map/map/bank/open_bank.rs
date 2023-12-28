use eo::{
    data::{i32, i32, i32, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub async fn open_bank(&mut self, player_id: i32, npc_index: i32) {
        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != EnfNpcType::Bank {
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

        let session_id = match player.generate_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to generate session id {}", e);
                return;
            }
        };

        player.set_interact_npc_index(npc_index);

        let mut builder = StreamBuilder::new();
        builder.add_int(character.gold_bank);
        builder.add_three(session_id as i32);
        builder.add_char(character.bank_level as i32);
        player.send(PacketAction::Open, PacketFamily::Bank, builder.get());
    }
}
