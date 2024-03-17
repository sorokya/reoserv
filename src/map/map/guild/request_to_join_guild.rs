use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            GuildReply, GuildReplyServerPacket, GuildReplyServerPacketReplyCodeData,
            GuildReplyServerPacketReplyCodeDataJoinRequest,
        },
        PacketAction, PacketFamily,
    },
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

        let recruiter = match self.characters.values().find(|c| c.name == recruiter_name) {
            Some(character) => character,
            None => {
                send_reply!(
                    character.player.as_ref().unwrap(),
                    GuildReply::RecruiterNotHere
                );
                return;
            }
        };

        if recruiter.guild_tag.is_none() {
            send_reply!(
                character.player.as_ref().unwrap(),
                GuildReply::RecruiterWrongGuild
            );
            return;
        }

        if let Some(tag) = &recruiter.guild_tag {
            if *tag != guild_tag {
                send_reply!(
                    character.player.as_ref().unwrap(),
                    GuildReply::RecruiterWrongGuild
                );
                return;
            }
        }

        if recruiter.guild_rank.unwrap() > 1 {
            send_reply!(character.player.as_ref().unwrap(), GuildReply::NotRecruiter);
            return;
        }

        recruiter
            .player
            .as_ref()
            .unwrap()
            .set_interact_player_id(Some(player_id));

        let packet = GuildReplyServerPacket {
            reply_code: GuildReply::JoinRequest,
            reply_code_data: Some(GuildReplyServerPacketReplyCodeData::JoinRequest(
                GuildReplyServerPacketReplyCodeDataJoinRequest {
                    player_id,
                    name: capitalize(&character.name),
                },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildReplyServerPacket: {}", e);
            return;
        }

        recruiter.player.as_ref().unwrap().send(
            PacketAction::Reply,
            PacketFamily::Guild,
            writer.to_byte_array(),
        );
    }
}
