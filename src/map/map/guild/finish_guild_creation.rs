use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{GuildAgreeServerPacket, GuildCreateServerPacket},
        PacketAction, PacketFamily,
    },
};

use crate::{character::Character, SETTINGS};

use super::super::Map;

impl Map {
    pub fn finish_guild_creation(
        &mut self,
        player_id: i32,
        member_ids: Vec<i32>,
        guild_tag: String,
        guild_name: String,
    ) {
        let mut guild_characters: Vec<Character> = Vec::with_capacity(SETTINGS.guild.min_players);

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

            guild_characters.push(character.to_owned());

            self.world.add_guild_member(player_id, guild_tag.clone());

            if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Create,
                    PacketFamily::Guild,
                    &GuildCreateServerPacket {
                        leader_player_id: player_id,
                        guild_tag: guild_tag.clone(),
                        guild_name: guild_name.clone(),
                        rank_name: SETTINGS.guild.default_leader_rank_name.clone(),
                        gold_amount: character.get_item_amount(1),
                    },
                );
            }
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

            guild_characters.push(character.to_owned());

            self.world.add_guild_member(*player_id, guild_tag.clone());

            if let Some(player) = character.player.as_ref() {
                player.send_buf(PacketAction::Agree, PacketFamily::Guild, buf.clone());
            }
        }

        let pool = self.pool.clone();

        tokio::spawn(async move {
            let mut conn = match pool.get_conn().await {
                Ok(conn) => conn,
                Err(e) => {
                    error!("Error getting connection from pool: {}", e);
                    return;
                }
            };

            for character in guild_characters.iter_mut() {
                character.save(&mut conn).await.unwrap_or_else(|e| {
                    error!("Error saving character: {}", e);
                });
            }
        });
    }
}
