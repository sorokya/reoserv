use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            client::GuildAgreeClientPacketInfoTypeData,
            server::{GuildReply, GuildReplyServerPacket},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use mysql_async::prelude::Queryable;
use mysql_common::params;

use crate::{utils::get_guild_ranks, NPC_DB, SETTINGS};

use super::{super::Player, validate_guild_description, validate_guild_rank};

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
    pub async fn update_guild(
        &mut self,
        session_id: i32,
        info_type_data: GuildAgreeClientPacketInfoTypeData,
    ) {
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

        match character.guild_rank {
            Some(rank_index) => {
                if rank_index > SETTINGS.guild.edit_rank {
                    return;
                }
            }
            None => return,
        }

        match info_type_data {
            GuildAgreeClientPacketInfoTypeData::Description(description) => {
                self.update_guild_description(
                    character.guild_tag.as_ref().unwrap(),
                    description.description,
                )
                .await
            }
            GuildAgreeClientPacketInfoTypeData::Ranks(ranks) => {
                self.update_guild_ranks(character.guild_tag.as_ref().unwrap(), ranks.ranks)
                    .await
            }
        }
    }

    pub async fn update_guild_description(&mut self, tag: &str, description: String) {
        if !validate_guild_description(&description) {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        match conn
            .exec_drop(
                "UPDATE Guild SET `description` = :description WHERE `tag` = :tag",
                params! {
                    "description" => description,
                    "tag" => tag,
                },
            )
            .await
        {
            Ok(_) => {
                send_reply!(self, GuildReply::Updated);
            }
            Err(e) => {
                error!("Error updating guild description: {}", e);
                return;
            }
        };
    }

    pub async fn update_guild_ranks(&mut self, tag: &str, ranks: [String; 9]) {
        if ranks.iter().any(|rank| !validate_guild_rank(rank)) {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        let existing_ranks = get_guild_ranks(&mut conn, tag).await;

        for (index, rank) in ranks.iter().enumerate() {
            if existing_ranks[index].eq(rank) {
                continue;
            }

            if let Err(e) = conn
                .exec_drop(
                    include_str!("../../../sql/update_guild_rank.sql"),
                    params! {
                        "rank" => rank,
                        "tag" => tag,
                        "index" => index + 1,
                    },
                )
                .await
            {
                error!("Error updating guild rank: {}", e);
                return;
            }
        }

        send_reply!(self, GuildReply::Updated);
    }
}
