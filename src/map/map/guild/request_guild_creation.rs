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
use mysql_async::{prelude::Queryable, Conn, Params};
use mysql_common::{params, Row};

use crate::NPC_DB;

use super::super::Map;

impl Map {
    pub fn request_guild_creation(
        &self,
        player_id: i32,
        npc_index: i32,
        guild_tag: String,
        guild_name: String,
    ) {
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

        let player = match character.player {
            Some(ref player) => player.clone(),
            None => return,
        };

        let pool = self.pool.clone();
        tokio::spawn(async move {
            let mut conn = pool.get_conn().await.unwrap();
            let packet = if check_guild_exists(&mut conn, &guild_tag, &guild_name).await {
                GuildReplyServerPacket {
                    reply_code: GuildReply::Exists,
                    reply_code_data: None,
                }
            } else {
                GuildReplyServerPacket {
                    reply_code: GuildReply::CreateBegin,
                    reply_code_data: None,
                }
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing GuildOpenServerPacket: {}", e);
                return;
            }

            player.send(
                PacketAction::Reply,
                PacketFamily::Guild,
                writer.to_byte_array(),
            );
        });
    }
}

async fn check_guild_exists(conn: &mut Conn, guild_tag: &str, guild_name: &str) -> bool {
    matches!(conn
        .exec_first::<Row, &str, Params>(
            "SELECT id FROM Guild WHERE name = :name OR tag = :tag",
            params! {
                "name" => guild_name,
                "tag" => guild_tag,
            },
        )
        .await, Ok(Some(_)))
}
