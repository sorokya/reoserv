use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{GuildReply, TalkServerServerPacket},
            PacketAction, PacketFamily,
        },
        r#pub::NpcType,
    },
};

use crate::{utils::get_guild_ranks, NPC_DB};

use super::super::Player;

impl Player {
    pub async fn assign_guild_rank(&mut self, session_id: i32, member_name: String, rank: i32) {
        if !(1..=9).contains(&rank) {
            return;
        }

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

        let mut conn = match self.pool.get_conn().await {
            Ok(conn) => conn,
            Err(e) => {
                error!("Error getting connection from pool: {}", e);
                return;
            }
        };

        let ranks = get_guild_ranks(&mut conn, guild_tag).await;
        let rank_str = match ranks.get(rank as usize - 1) {
            Some(rank) => rank,
            None => return,
        };

        let target_character = match self
            .world
            .get_character_by_name(member_name.to_owned())
            .await
        {
            Ok(character) => character,
            Err(_) => {
                let packet = TalkServerServerPacket {
                    message: "Offline rank updating not currently supported".to_owned(),
                };

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Error serializing TalkServerServerPacket: {}", e);
                    return;
                }

                let _ = self
                    .bus
                    .send(
                        PacketAction::Server,
                        PacketFamily::Talk,
                        writer.to_byte_array(),
                    )
                    .await;

                return;
                // TODO: handle offline
                /* self.assign_guild_rank_offline(guild_tag, member_name, rank, rank_str)
                    .await;
                return; */
            }
        };

        let target_guild_tag = match target_character.guild_tag {
            Some(ref guild_tag) => guild_tag,
            None => return,
        };

        if guild_tag != target_guild_tag {
            send_reply!(self, GuildReply::RankingNotMember);
            return;
        }

        if target_character.is_guild_leader() {
            send_reply!(self, GuildReply::RankingLeader);
            return;
        }

        let map = match self.world.get_map(target_character.map_id).await {
            Ok(map) => map,
            Err(e) => {
                error!("Error getting map: {}", e);
                return;
            }
        };

        map.update_guild_rank(target_character.player_id.unwrap(), rank, rank_str);

        send_reply!(self, GuildReply::Updated);
    }
}
