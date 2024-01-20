use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            client::GuildInfoType,
            server::{GuildRankServerPacket, GuildSellServerPacket, GuildTakeServerPacket},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};
use mysql_async::{prelude::Queryable, Conn, Params};
use mysql_common::{params, Row};

use crate::NPC_DB;

use super::super::Player;

impl Player {
    pub async fn request_guild_info(&mut self, session_id: i32, info_type: GuildInfoType) {
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

        let character = match map.get_character(self.id).await {
            Some(character) => character,
            None => return,
        };

        if character.guild_tag.is_none() {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        let mut writer = EoWriter::new();

        match info_type {
            GuildInfoType::Description => {
                let description =
                    get_guild_description(&mut conn, character.guild_tag.as_ref().unwrap()).await;
                let packet = GuildTakeServerPacket { description };

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Error serializing GuildTakeServerPacket: {}", e);
                    return;
                }

                let _ = self
                    .bus
                    .send(
                        PacketAction::Take,
                        PacketFamily::Guild,
                        writer.to_byte_array(),
                    )
                    .await;
            }
            GuildInfoType::Ranks => {
                let ranks = get_guild_ranks(&mut conn, character.guild_tag.as_ref().unwrap()).await;

                let packet = GuildRankServerPacket {
                    ranks: [
                        ranks[0].to_owned(),
                        ranks[1].to_owned(),
                        ranks[2].to_owned(),
                        ranks[3].to_owned(),
                        ranks[4].to_owned(),
                        ranks[5].to_owned(),
                        ranks[6].to_owned(),
                        ranks[7].to_owned(),
                        ranks[8].to_owned(),
                    ],
                };

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Error serializing GuildRankServerPacket: {}", e);
                    return;
                }

                let _ = self
                    .bus
                    .send(
                        PacketAction::Rank,
                        PacketFamily::Guild,
                        writer.to_byte_array(),
                    )
                    .await;
            }
            GuildInfoType::Bank => {
                let gold_amount =
                    get_guild_bank(&mut conn, character.guild_tag.as_ref().unwrap()).await;
                let packet = GuildSellServerPacket { gold_amount };

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Error serializing GuildSellServerPacket: {}", e);
                    return;
                }

                let _ = self
                    .bus
                    .send(
                        PacketAction::Sell,
                        PacketFamily::Guild,
                        writer.to_byte_array(),
                    )
                    .await;
            }
            _ => {}
        };
    }
}

async fn get_guild_description(conn: &mut Conn, tag: &str) -> String {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `description` FROM Guild WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => row.get(0).unwrap(),
        Err(e) => {
            error!("Error getting guild description: {}", e);
            "".to_owned()
        }
        _ => "".to_owned(),
    }
}

async fn get_guild_ranks(conn: &mut Conn, tag: &str) -> Vec<String> {
    match conn
        .exec_map(
            "SELECT `rank` FROM Guild INNER JOIN GuildRank ON GuildRank.`guild_id` = Guild.`id` WHERE `tag` = :tag ORDER BY `index` ASC",
            params! {
                "tag" => tag,
            },
            |row: Row| row.get::<String, usize>(0).unwrap(),
        )
        .await
    {
        Ok(ranks) => ranks,
        Err(e) => {
            error!("Error getting guild ranks: {}", e);
            vec!["".to_owned(); 9]
        }
    }
}

async fn get_guild_bank(conn: &mut Conn, tag: &str) -> i32 {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `bank` FROM Guild WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => row.get::<i32, usize>(0).unwrap(),
        Err(e) => {
            error!("Error getting guild bank: {}", e);
            0
        }
        _ => 0,
    }
}
