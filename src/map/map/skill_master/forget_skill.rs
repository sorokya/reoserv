use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{server::StatSkillRemoveServerPacket, PacketAction, PacketFamily},
        r#pub::NpcType,
    },
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub async fn forget_skill(&mut self, player_id: i32, skill_id: i32, session_id: i32) {
        if skill_id == 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.has_spell(skill_id) {
            return;
        }

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

        if npc_data.r#type != NpcType::Trainer {
            return;
        }

        character.remove_spell(skill_id);

        let reply = StatSkillRemoveServerPacket { spell_id: skill_id };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize packet {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Remove,
            PacketFamily::StatSkill,
            writer.to_byte_array(),
        );
    }
}
