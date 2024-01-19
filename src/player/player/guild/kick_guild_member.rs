use crate::SETTINGS;

use super::super::Player;
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{GuildReply, GuildReplyServerPacket},
        PacketAction, PacketFamily,
    },
};

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
    pub async fn kick_guild_member(&mut self, session_id: i32, member_name: String) {
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

        match character.guild_rank_index {
            Some(rank_index) => {
                if rank_index > SETTINGS.guild.kick_rank {
                    return;
                }
            }
            None => return,
        };

        match self.world.get_character_by_name(member_name.clone()).await {
            Ok(member) => {
                if member.guild_tag != character.guild_tag {
                    send_reply!(self, GuildReply::RemoveNotMember);
                    return;
                }

                if member.guild_rank_index.unwrap() <= character.guild_rank_index.unwrap() {
                    send_reply!(self, GuildReply::RemoveLeader);
                    return;
                }

                let member_map = match self.world.get_map(member.map_id).await {
                    Ok(map) => map,
                    Err(_) => {
                        error!("Error getting map {}", member.map_id);
                        return;
                    }
                };

                member_map.kick_from_guild(member.player_id.unwrap());

                // TODO: Guild announce
                // self.world.guild_announcement(guild_id, "Blah was kicked from the guild")

                send_reply!(self, GuildReply::Removed);
            }
            Err(_) => {
                // TODO: Offline kick
            }
        }
    }
}
