use eolib::{
    protocol::{
        net::{
            server::{GuildMember, GuildReply, GuildTellServerPacket},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};
use mysql_async::prelude::Queryable;
use mysql_common::{params, Row};

use crate::NPC_DB;

use super::super::Player;

impl Player {
    pub async fn request_guild_memberlist(&mut self, session_id: i32, guild_identity: String) {
        let npc_index = match self.interact_npc_index {
            Some(npc_index) => npc_index,
            None => return,
        };

        match self.session_id {
            Some(id) => {
                if id != session_id {
                    return;
                }
            }
            None => return,
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

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        let members: Vec<GuildMember> = match conn
            .exec_map(
                include_str!("../../../sql/get_guild_memberlist.sql"),
                params! {
                    "guild_identity" => &guild_identity,
                },
                |mut row: Row| GuildMember {
                    rank: row.take("guild_rank").unwrap(),
                    name: row.take("name").unwrap(),
                    rank_name: row.take("guild_rank_string").unwrap(),
                },
            )
            .await
        {
            Ok(members) => members,
            Err(e) => {
                error!("Error getting guild memberlist: {}", e);
                return;
            }
        };

        if members.is_empty() {
            send_reply!(self, GuildReply::NotFound);
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Tell,
                PacketFamily::Guild,
                GuildTellServerPacket { members },
            )
            .await;
    }
}
