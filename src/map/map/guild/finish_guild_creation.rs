use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{GuildAgreeServerPacket, GuildCreateServerPacket},
        PacketAction, PacketFamily,
    },
};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn finish_guild_creation(
        &mut self,
        player_id: i32,
        member_ids: Vec<i32>,
        guild_tag: String,
        guild_name: String,
    ) {
        {
            let character = match self.characters.get_mut(&player_id) {
                Some(character) => character,
                None => return,
            };

            character.remove_item(1, SETTINGS.guild.create_cost);
            character.guild_tag = Some(guild_tag.clone());
            character.guild_name = Some(guild_name.clone());
            character.guild_rank_string = Some(SETTINGS.guild.default_leader_rank_name.clone());
            character.guild_rank = Some(1);

            let packet = GuildCreateServerPacket {
                leader_player_id: player_id,
                guild_tag: guild_tag.clone(),
                guild_name: guild_name.clone(),
                rank_name: SETTINGS.guild.default_leader_rank_name.clone(),
                gold_amount: character.get_item_amount(1),
            };

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Error serializing GuildCreateServerPacket: {}", e);
                return;
            }

            character.player.as_ref().unwrap().send(
                PacketAction::Create,
                PacketFamily::Guild,
                writer.to_byte_array(),
            );
        }

        let packet = GuildAgreeServerPacket {
            recruiter_id: player_id,
            guild_tag: guild_tag.clone(),
            guild_name: guild_name.clone(),
            rank_name: SETTINGS.guild.default_new_member_rank_name.clone(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Error serializing GuildAgreeServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for player_id in &member_ids {
            let character = match self.characters.get_mut(player_id) {
                Some(character) => character,
                None => continue,
            };

            if character.guild_tag.is_some() {
                continue;
            }

            character.guild_tag = Some(guild_tag.clone());
            character.guild_name = Some(guild_name.clone());
            character.guild_rank_string = Some(SETTINGS.guild.default_new_member_rank_name.clone());
            character.guild_rank = Some(9);

            character.player.as_ref().unwrap().send(
                PacketAction::Agree,
                PacketFamily::Guild,
                buf.clone(),
            );
        }
    }
}
