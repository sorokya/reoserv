use eolib::protocol::{
    net::{server::StatSkillRemoveServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub fn forget_skill(&mut self, player_id: i32, npc_index: i32, skill_id: i32) {
        if skill_id <= 0 {
            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !character.has_spell(skill_id) {
            return;
        }

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

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Remove,
                PacketFamily::StatSkill,
                &StatSkillRemoveServerPacket { spell_id: skill_id },
            );
        }
    }
}
