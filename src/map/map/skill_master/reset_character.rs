use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::statskill::Junk, PacketAction, PacketFamily},
    pubs::EnfNpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub async fn reset_character(&mut self, player_id: EOShort, session_id: EOShort) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        let actual_session_id = match character.player.as_ref().unwrap().get_session_id().await {
            Ok(id) => id,
            Err(e) => {
                error!("Failed to get session id {}", e);
                return;
            }
        };

        if actual_session_id != session_id {
            return;
        }

        let npc_index = match character
            .player
            .as_ref()
            .unwrap()
            .get_interact_npc_index()
            .await
        {
            Some(index) => index,
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

        if npc_data.r#type != EnfNpcType::Skills {
            return;
        }

        character.reset();

        let reply = Junk {
            stats: character.get_character_stats_1(),
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        character.player.as_ref().unwrap().send(
            PacketAction::Junk,
            PacketFamily::StatSkill,
            builder.get(),
        );
    }
}
