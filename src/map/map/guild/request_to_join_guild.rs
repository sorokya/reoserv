use eolib::protocol::net::{
    server::{
        GuildReply, GuildReplyServerPacket, GuildReplyServerPacketReplyCodeData,
        GuildReplyServerPacketReplyCodeDataJoinRequest,
    },
    PacketAction, PacketFamily,
};

use crate::utils::capitalize;

use super::super::Map;

impl Map {
    pub fn request_to_join_guild(&self, player_id: i32, guild_tag: String, recruiter_name: String) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.guild_tag.is_some() {
            return;
        }

        let player = match character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        let recruiter = match self.characters.values().find(|c| c.name == recruiter_name) {
            Some(character) => character,
            None => {
                send_reply!(player, GuildReply::RecruiterNotHere);
                return;
            }
        };

        if recruiter.guild_tag.is_none() {
            send_reply!(player, GuildReply::RecruiterWrongGuild);
            return;
        }

        if let Some(tag) = &recruiter.guild_tag {
            if *tag != guild_tag {
                send_reply!(player, GuildReply::RecruiterWrongGuild);
                return;
            }
        }

        if recruiter.guild_rank.unwrap_or(9) > 1 {
            send_reply!(player, GuildReply::NotRecruiter);
            return;
        }

        if let Some(player) = recruiter.player.as_ref() {
            player.set_interact_player_id(Some(player_id));
            player.send(
                PacketAction::Reply,
                PacketFamily::Guild,
                &GuildReplyServerPacket {
                    reply_code: GuildReply::JoinRequest,
                    reply_code_data: Some(GuildReplyServerPacketReplyCodeData::JoinRequest(
                        GuildReplyServerPacketReplyCodeDataJoinRequest {
                            player_id,
                            name: capitalize(&character.name),
                        },
                    )),
                },
            );
        }
    }
}
