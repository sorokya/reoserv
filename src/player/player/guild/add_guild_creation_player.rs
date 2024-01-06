use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            GuildReply, GuildReplyServerPacket, GuildReplyServerPacketReplyCodeData,
            GuildReplyServerPacketReplyCodeDataCreateAdd,
            GuildReplyServerPacketReplyCodeDataCreateAddConfirm,
        },
        PacketAction, PacketFamily,
    },
};

use crate::SETTINGS;

use super::super::Player;

impl Player {
    pub async fn add_guild_creation_player(&mut self, player_id: i32, name: String) {
        if self.guild_create_members.contains(&player_id) {
            return;
        }

        self.guild_create_members.push(player_id);

        let packet = if self.guild_create_members.len() + 1 >= SETTINGS.guild.min_players {
            GuildReplyServerPacket {
                reply_code: GuildReply::CreateAddConfirm,
                reply_code_data: Some(GuildReplyServerPacketReplyCodeData::CreateAddConfirm(
                    GuildReplyServerPacketReplyCodeDataCreateAddConfirm { name },
                )),
            }
        } else {
            GuildReplyServerPacket {
                reply_code: GuildReply::CreateAdd,
                reply_code_data: Some(GuildReplyServerPacketReplyCodeData::CreateAdd(
                    GuildReplyServerPacketReplyCodeDataCreateAdd { name },
                )),
            }
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
    }
}
