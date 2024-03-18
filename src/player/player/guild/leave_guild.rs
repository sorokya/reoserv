use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{GuildAcceptServerPacket, GuildAgreeServerPacket},
        PacketAction, PacketFamily,
    },
};
use mysql_async::prelude::Queryable;
use mysql_common::{params, Row};

use super::super::Player;

impl Player {
    pub async fn leave_guild(&mut self, session_id: i32) {
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

        let character = match map.get_character(self.id).await {
            Some(character) => character,
            None => return,
        };

        let guild_tag = match character.guild_tag {
            Some(ref tag) => tag,
            None => return,
        };

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        let leader_count = match conn
            .exec_map(
                include_str!("../../../sql/get_count_guild_leader.sql"),
                params! {
                    "guild_tag" => guild_tag,
                },
                |mut row: Row| row.take::<i32, usize>(0).unwrap(),
            )
            .await
        {
            Ok(leader_counts) => match leader_counts.first() {
                Some(leader_count) => *leader_count,
                None => 0,
            },
            Err(e) => {
                error!("Error getting leader count: {}", e);
                return;
            }
        };

        if leader_count == 1 {
            self.send_server_message("You are the last leader and cannot leave the guild. You must promote someone else to leader first.")
                .await;

            // This is dumb but it tricks the v28 client into keeping you in your guild
            let packet = GuildAgreeServerPacket {
                recruiter_id: self.id,
                guild_tag: guild_tag.to_owned(),
                guild_name: character.guild_name.unwrap().clone(),
                rank_name: character.guild_rank_string.unwrap().clone(),
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing GuildAgreeServerPacket: {}", e);
                return;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Agree,
                    PacketFamily::Guild,
                    writer.to_byte_array(),
                )
                .await;

            let packet = GuildAcceptServerPacket { rank: 1 };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing GuildAcceptServerPacket: {}", e);
                return;
            }

            let _ = self
                .bus
                .send(
                    PacketAction::Accept,
                    PacketFamily::Guild,
                    writer.to_byte_array(),
                )
                .await;

            return;
        }

        let map = match self.world.get_map(character.map_id).await {
            Ok(map) => map,
            Err(e) => {
                error!("Error getting map: {}", e);
                return;
            }
        };

        map.leave_guild(self.id);
    }
}
