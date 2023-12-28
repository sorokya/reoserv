use eo::{
    data::{i32, EOShort, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::{INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub async fn request_citizenship(
        &mut self,
        player_id: EOShort,
        session_id: EOShort,
        answers: [String; 3],
    ) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let actual_session_id = match player.get_session_id().await {
            Ok(session_id) => session_id,
            Err(_) => return,
        };

        if session_id != actual_session_id {
            return;
        }

        let npc_index = match player.get_interact_npc_index().await {
            Some(npc_index) => npc_index,
            None => return,
        };

        let npc = match self.npcs.get(&npc_index) {
            Some(npc) => npc,
            None => return,
        };

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

        let mut wrong_answers = 0;
        if answers[0] != inn_data.answer1 {
            wrong_answers += 1;
        }

        if answers[1] != inn_data.answer2 {
            wrong_answers += 1;
        }

        if answers[2] != inn_data.answer3 {
            wrong_answers += 1;
        }

        if wrong_answers == 0 {
            character.home = inn_data.name.clone();
        }

        let mut builder = StreamBuilder::new();
        builder.add_char(wrong_answers as i32);

        player.send(PacketAction::Reply, PacketFamily::Citizen, builder.get());
    }
}
