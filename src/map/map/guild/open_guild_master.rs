use eolib::protocol::{
    net::{server::GuildOpenServerPacket, PacketAction, PacketFamily},
    r#pub::NpcType,
};

use crate::{utils::in_client_range, NPC_DB};

use super::super::Map;

impl Map {
    pub fn open_guild_master(&self, player_id: i32, npc_index: i32) {
        let character = match self.characters.get(&player_id) {
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

        if npc_data.r#type != NpcType::Guild {
            return;
        }

        if !in_client_range(&character.coords, &npc.coords) {
            return;
        }

        let player = match character.player {
            Some(ref player) => player.clone(),
            None => return,
        };

        tokio::spawn(async move {
            let session_id = match player.generate_session_id().await {
                Ok(session_id) => session_id,
                Err(e) => {
                    error!("Error generating session id: {}", e);
                    return;
                }
            };

            player.set_interact_npc_index(npc_index);

            player.send(
                PacketAction::Open,
                PacketFamily::Guild,
                &GuildOpenServerPacket { session_id },
            );
        });
    }
}
