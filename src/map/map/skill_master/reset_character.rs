use eolib::protocol::{
    net::{server::StatSkillJunkServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub async fn reset_character(&mut self, player_id: i32, session_id: i32) {
        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        {
            let player = match character.player.as_ref() {
                Some(player) => player,
                None => return,
            };

            let actual_session_id = match player.get_session_id().await {
                Ok(id) => id,
                Err(e) => {
                    error!("Failed to get session id {}", e);
                    return;
                }
            };

            if actual_session_id != session_id {
                return;
            }

            let npc_index = match player.get_interact_npc_index().await {
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

            if npc_data.r#type != NpcType::Trainer {
                return;
            }
        }

        character.reset();

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Junk,
                PacketFamily::StatSkill,
                &StatSkillJunkServerPacket {
                    stats: character.get_character_stats_reset(),
                },
            );
        }
    }
}
