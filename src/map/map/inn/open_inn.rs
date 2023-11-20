use eo::{
    data::{EOChar, EOShort, EOThree, StreamBuilder, EO_BREAK_CHAR},
    protocol::{PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::{utils::in_client_range, INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub async fn open_inn(&self, player_id: EOShort, npc_index: EOChar) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

        if !in_client_range(&character.coords, &npc.coords) {
            return;
        }

        let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != EnfNpcType::Inn {
            return;
        }

        let inn_data = match INN_DB
            .inns
            .iter()
            .find(|inn| inn.vendor_id == npc_data.behavior_id)
        {
            Some(inn_data) => inn_data,
            None => return,
        };

        let current_inn_data = match INN_DB.inns.iter().find(|inn| inn.name == character.home) {
            Some(inn_data) => inn_data,
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
        builder.add_three(inn_data.vendor_id as EOThree + 1);
        builder.add_char(current_inn_data.vendor_id as EOChar - 1);
        builder.add_short(session_id);
        builder.add_byte(EO_BREAK_CHAR);
        builder.add_break_string(&inn_data.question1);
        builder.add_break_string(&inn_data.question2);
        builder.add_break_string(&inn_data.question3);

        player.send(PacketAction::Open, PacketFamily::Citizen, builder.get());
    }
}
