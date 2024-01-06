use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{GuildReply, GuildReplyServerPacket},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use crate::{
    player::player::guild::{validate_guild_name, validate_guild_tag},
    utils::capitalize,
    NPC_DB, SETTINGS,
};

use super::{super::Player, guild_exists};

macro_rules! send_reply {
    ($player:expr, $reply:expr) => {{
        let mut writer = EoWriter::new();
        let packet = GuildReplyServerPacket {
            reply_code: $reply,
            reply_code_data: None,
        };

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildReplyServerPacket: {}", e);
            return;
        }

        let _ = $player
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Guild,
                writer.to_byte_array(),
            )
            .await;
    }};
}

impl Player {
    pub async fn request_guild_creation(
        &mut self,
        session_id: i32,
        guild_name: String,
        guild_tag: String,
    ) {
        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        let actual_session_id = match self.session_id {
            Some(session_id) => session_id,
            None => {
                return;
            }
        };

        if session_id != actual_session_id {
            return;
        }

        if !validate_guild_tag(&guild_tag) || !validate_guild_name(&guild_name) {
            send_reply!(self, GuildReply::NotApproved);
            return;
        }

        let map = match self.map {
            Some(ref map) => map,
            None => return,
        };

        let npc_id = match map.get_npc_id_for_index(npc_index).await {
            Some(npc_id) => npc_id,
            None => return,
        };

        let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
            Some(npc_data) => npc_data,
            None => return,
        };

        if npc_data.r#type != NpcType::Guild {
            return;
        }

        let character = match map.get_character(self.id).await {
            Some(character) => character,
            None => return,
        };

        if character.guild_tag.is_some()
            || character.get_item_amount(1) < SETTINGS.guild.create_cost
        {
            return;
        }

        let mut conn = self.pool.get_conn().await.unwrap();

        if guild_exists(&mut conn, &guild_tag, &guild_name).await {
            send_reply!(self, GuildReply::Exists);
            return;
        }

        self.guild_create_members = Vec::with_capacity(SETTINGS.guild.min_players);

        send_reply!(self, GuildReply::CreateBegin);

        map.send_guild_create_requests(
            self.id,
            format!(
                "{} ({})",
                capitalize(&character.name.to_lowercase()),
                guild_tag.to_uppercase()
            ),
        );
    }
}
