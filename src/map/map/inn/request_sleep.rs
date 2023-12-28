use eolib::{protocol::{r#pub::NpcType, net::{PacketAction, PacketFamily}}, data::EoWriter};

use crate::{INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub async fn request_sleep(&mut self, player_id: i32, session_id: i32) {
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

        if npc_data.r#type != NpcType::Inn {
            return;
        }

        let inn_data = match INN_DB
            .inns
            .iter()
            .find(|inn| inn.behavior_id == npc_data.behavior_id)
        {
            Some(inn) => inn,
            None => return,
        };

        if inn_data.name != character.home {
            return;
        }

        let cost = (character.max_hp - character.hp) + (character.max_tp - character.tp);

        let mut writer = EoWriter::new();
        writer.add_int(cost);

        player.set_sleep_cost(cost);
        player.send(PacketAction::Request, PacketFamily::Citizen, writer.to_byte_array());
    }
}
