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

        let player = match character.player.as_ref() {
            Some(player) => player.to_owned(),
            None => return,
        };

        if character.guild_tag.is_some() {
            send_reply!(player, GuildReply::AlreadyMember);
            return;
        }

        let recruiter = match self.characters.values().find(|c| c.name == recruiter_name) {
            Some(character) => Some(character.to_owned()),
            None => None,
        };

        let world = self.world.to_owned();
        let character_name = character.name.to_owned();

        tokio::spawn(async move {
            match world.get_character_by_name(recruiter_name.to_owned()).await {
                Ok(_) => {}
                _ => {
                    send_reply!(player, GuildReply::RecruiterOffline);
                    return;
                }
            }

            let recruiter = match recruiter {
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
                                name: capitalize(&character_name),
                            },
                        )),
                    },
                );
            }
        });
    }
}
