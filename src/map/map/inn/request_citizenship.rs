use eolib::protocol::{
    net::{server::CitizenReplyServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::{INN_DB, NPC_DB};

use super::super::Map;

impl Map {
    pub fn request_citizenship(&mut self, player_id: i32, npc_index: i32, answers: [String; 3]) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
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
        if answers[0] != inn_data.questions[0].answer {
            questions_wrong += 1;
        }

        if answers[1] != inn_data.questions[1].answer {
            questions_wrong += 1;
        }

        if answers[2] != inn_data.questions[2].answer {
            questions_wrong += 1;
        }

        if questions_wrong == 0 {
            character.home = inn_data.name.clone();
        }

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Reply,
                PacketFamily::Citizen,
                &CitizenReplyServerPacket { questions_wrong },
            );
        }
    }
}
