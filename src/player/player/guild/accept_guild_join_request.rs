use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{GuildReply, GuildReplyServerPacket},
        PacketAction, PacketFamily,
    },
};
use mysql_async::{prelude::Queryable, Conn, Params};
use mysql_common::{params, Row};

use crate::SETTINGS;

use super::super::Player;

impl Player {
    pub async fn accept_guild_join_request(&mut self, player_id: i32) {
        match self.interact_player_id {
            Some(id) => {
                if id != player_id {
                    return;
                }
            }
            None => return,
        }

        self.interact_player_id = None;

        let map = match self.map {
            Some(ref map) => map,
            None => return,
        };

        let character = match map.get_character(self.id).await {
            Some(character) => character,
            None => return,
        };

        let tag = match character.guild_tag {
            Some(ref tag) => tag,
            None => return,
        };

        if character.guild_rank_index.unwrap() > 1 {
            return;
        }

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        let guild_bank = get_guild_bank(&mut conn, character.guild_tag.as_ref().unwrap()).await;
        if guild_bank < SETTINGS.guild.recruit_cost {
            let packet = GuildReplyServerPacket {
                reply_code: GuildReply::AccountLow,
                reply_code_data: None,
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing GuildReplyServerPacket: {}", e);
                return;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Reply,
                    PacketFamily::Guild,
                    writer.to_byte_array(),
                )
                .await;
            return;
        }

        if let Err(e) =
            set_guild_bank(&mut conn, tag, guild_bank - SETTINGS.guild.recruit_cost).await
        {
            error!("Error setting guild bank: {}", e);
            return;
        }

        let guild_name = match get_guild_name(&mut conn, tag).await {
            Some(guild_name) => guild_name,
            None => return,
        };

        let rank_string = match get_new_member_guild_rank(&mut conn, tag).await {
            Some(rank_string) => rank_string,
            None => return,
        };

        map.join_guild(player_id, self.id, tag.to_owned(), guild_name, rank_string);
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

async fn set_guild_bank(conn: &mut Conn, tag: &str, bank: i32) -> Result<(), mysql_async::Error> {
    conn.exec_drop(
        "UPDATE Guild SET `bank` = :bank WHERE `tag` = :tag",
        params! {
            "bank" => bank,
            "tag" => tag,
        },
    )
    .await
}

async fn get_guild_name(conn: &mut Conn, tag: &str) -> Option<String> {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `name` FROM Guild WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => Some(row.get(0).unwrap()),
        Err(e) => {
            error!("Error getting guild name: {}", e);
            None
        }
        _ => None,
    }
}

async fn get_new_member_guild_rank(conn: &mut Conn, tag: &str) -> Option<String> {
    match conn
        .exec_first::<Row, &str, Params>(
            "SELECT `rank` FROM Guild INNER JOIN GuildRank ON GuildRank.`guild_id` = Guild.`id` AND GuildRank.`index` = 8 WHERE `tag` = :tag",
            params! {
                "tag" => tag,
            },
        )
        .await
    {
        Ok(Some(row)) => Some(row.get(0).unwrap()),
        Err(e) => {
            error!("Error getting guild rank: {}", e);
            None
        }
        _ => None,
    }
}
