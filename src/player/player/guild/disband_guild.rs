use eolib::protocol::r#pub::NpcType;

use crate::NPC_DB;

use super::super::Player;

impl Player {
    pub async fn disband_guild(&mut self, session_id: i32) {
        match self.session_id {
            Some(id) => {
                if id != session_id {
                    return;
                }
            }
            None => return,
        }

        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        let map = match self.map {
            Some(ref map) => map,
            None => return,
        };

        let npc_id = match map.get_npc_id_for_index(npc_index).await {
            Some(npc_id) => npc_id,
            None => return,
        };

        match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => {
                if npc_data.r#type != NpcType::Guild {
                    return;
                }
            }
            None => return,
        };

        let character = match map.get_character(self.id).await {
            Some(character) => character,
            None => {
                return;
            }
        };

        if !character.is_guild_leader() {
            return;
        }

        let guild_tag = match character.guild_tag {
            Some(ref guild_tag) => guild_tag,
            None => return,
        };

        self.world.disband_guild(guild_tag.to_owned());
    }
}
