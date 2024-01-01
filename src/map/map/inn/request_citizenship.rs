use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::CitizenReplyServerPacket, PacketAction, PacketFamily},
        r#pub::NpcType,
    },
};

use crate::{INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub async fn request_citizenship(
        &mut self,
        player_id: i32,
        session_id: i32,
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

        let mut questions_wrong = 0;
        if answers[0] != inn_data.answer1 {
            questions_wrong += 1;
        }

        if answers[1] != inn_data.answer2 {
            questions_wrong += 1;
        }

        if answers[2] != inn_data.answer3 {
            questions_wrong += 1;
        }

        if questions_wrong == 0 {
            character.home = inn_data.name.clone();
        }

        let packet = CitizenReplyServerPacket { questions_wrong };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize CitizenReplyServerPacket: {}", e);
            return;
        }

        player.send(
            PacketAction::Reply,
            PacketFamily::Citizen,
            writer.to_byte_array(),
        );
    }
}
