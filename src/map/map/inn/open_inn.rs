use eolib::{
    data::EoWriter,
    protocol::{
        net::{PacketAction, PacketFamily},
        r#pub::NpcType,
    },
};

use crate::{utils::in_client_range, INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub async fn open_inn(&self, player_id: i32, npc_index: i32) {
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

        if npc_data.r#type != NpcType::Inn {
            return;
        }

        let inn_data = match INN_DB
            .inns
            .iter()
            .find(|inn| inn.behavior_id == npc_data.behavior_id)
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

        let mut writer = EoWriter::new();
        writer.add_three(inn_data.behavior_id + 1);
        writer.add_char(current_inn_data.behavior_id - 1);
        writer.add_short(session_id);
        writer.add_byte(0xff);
        writer.add_string(&inn_data.question1);
        writer.add_byte(0xff);
        writer.add_string(&inn_data.question2);
        writer.add_byte(0xff);
        writer.add_string(&inn_data.question3);
        writer.add_byte(0xff);

        player.send(
            PacketAction::Open,
            PacketFamily::Citizen,
            writer.to_byte_array(),
        );
    }
}
