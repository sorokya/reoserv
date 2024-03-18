use chrono::NaiveDateTime;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{GuildReply, GuildReportServerPacket, GuildStaff},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};
use futures::StreamExt;
use mysql_async::prelude::Queryable;
use mysql_common::{params, Row};

use crate::NPC_DB;

use super::super::Player;

impl Player {
    pub async fn request_guild_details(&mut self, session_id: i32, guild_identity: String) {
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

        let mut result = match conn
            .exec_iter(
                "CALL GetGuildDetails(:guild_identity);",
                params! {
                    "guild_identity" => &guild_identity,
                },
            )
            .await
        {
            Ok(result) => result,
            Err(e) => {
                error!("Error getting guild details: {}", e);
                return;
            }
        };

        let mut packet = GuildReportServerPacket::default();

        {
            let mut stream = match result.stream::<Row>().await {
                Ok(Some(stream)) => stream,
                Ok(None) => {
                    send_reply!(self, GuildReply::NotFound);
                    return;
                }
                Err(e) => {
                    error!("Error getting guild details: {}", e);
                    return;
                }
            };

            let mut row = match stream.next().await {
                Some(Ok(row)) => row,
                Some(Err(e)) => {
                    error!("Error getting guild details: {}", e);
                    return;
                }
                None => {
                    error!("Error getting guild details: no rows returned");
                    return;
                }
            };

            packet.tag = row.take("tag").unwrap();
            packet.name = row.take("name").unwrap();
            packet.description = row.take("description").unwrap();

            let created_at: NaiveDateTime = row.take("created_at").unwrap();
            packet.create_date = created_at.format("%Y-%m-%d").to_string();

            let bank: i32 = row.take("bank").unwrap();
            packet.wealth = if bank < 2000 {
                "bankrupt".to_string()
            } else if bank < 10_000 {
                "poor".to_string()
            } else if bank < 50_000 {
                "normal".to_string()
            } else if bank < 100_000 {
                "wealthy".to_string()
            } else {
                "very wealthy".to_string()
            };
        }

        {
            let mut stream = match result.stream::<Row>().await {
                Ok(Some(stream)) => stream,
                Ok(None) => {
                    send_reply!(self, GuildReply::NotFound);
                    return;
                }
                Err(e) => {
                    error!("Error getting guild details: {}", e);
                    return;
                }
            };

            let mut index = 0;
            while let Some(row) = stream.next().await {
                let mut row = match row {
                    Ok(row) => row,
                    Err(e) => {
                        error!("Error getting guild details: {}", e);
                        return;
                    }
                };

                // Client won't display ranks less than 4 characters long
                packet.ranks[index] = format!("{:<4}", row.take::<String, &str>("rank").unwrap());

                index += 1;
            }
        }

        {
            let mut stream = match result.stream::<Row>().await {
                Ok(Some(stream)) => stream,
                Ok(None) => {
                    send_reply!(self, GuildReply::NotFound);
                    return;
                }
                Err(e) => {
                    error!("Error getting guild details: {}", e);
                    return;
                }
            };

            while let Some(row) = stream.next().await {
                let mut row = match row {
                    Ok(row) => row,
                    Err(e) => {
                        error!("Error getting guild details: {}", e);
                        return;
                    }
                };

                packet.staff.push(GuildStaff {
                    rank: row.take("guild_rank").unwrap(),
                    name: row.take("name").unwrap(),
                });
            }
        }

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildReportServerPacket: {}", e);
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Report,
                PacketFamily::Guild,
                writer.to_byte_array(),
            )
            .await;
    }
}
