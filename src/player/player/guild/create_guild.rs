use eolib::{protocol::net::server::GuildReply};
use mysql_async::{prelude::Queryable, Conn};
use mysql_common::params;

use crate::SETTINGS;

use super::{
    super::Player, guild_exists, validate_guild_description, validate_guild_name,
    validate_guild_tag,
};

impl Player {
    pub async fn create_guild(
        &mut self,
        session_id: i32,
        guild_name: String,
        guild_tag: String,
        guild_description: String,
    ) {
        if self.guild_create_members.len() + 1 < SETTINGS.guild.min_players {
            return;
        }

        let actual_session_id = match self.session_id {
            Some(session_id) => session_id,
            None => {
                return;
            }
        };

        if session_id != actual_session_id {
            return;
        }

        if !validate_guild_tag(&guild_tag)
            || !validate_guild_name(&guild_name)
            || !validate_guild_description(&guild_description)
        {
            send_reply!(self, GuildReply::NotApproved);
            return;
        }

        let map = match self.map {
            Some(ref map) => map,
            None => return,
        };

        let character = match map.get_character(self.id).await {
            Some(character) => character,
            None => return,
        };

        if character.guild_tag.is_some()
            || character.get_item_amount(1) < SETTINGS.guild.create_cost
        {
            return;
        }

        let mut conn = self.pool.get_conn().await.unwrap();

        if guild_exists(&mut conn, &guild_tag, &guild_name).await {
            send_reply!(self, GuildReply::Exists);
            return;
        }

        if let Err(e) = create_guild(&mut conn, &guild_tag, &guild_name, &guild_description).await {
            error!("Error creating guild: {}", e);
            return;
        }

        map.finish_guild_creation(
            self.id,
            self.guild_create_members.clone(),
            guild_tag,
            guild_name,
        );
    }
}

async fn create_guild(
    conn: &mut Conn,
    tag: &str,
    name: &str,
    description: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    conn.exec_drop(
        include_str!("../../../sql/create_guild.sql"),
        params! {
            "tag" => tag,
            "name" => name,
            "description" => description,
        },
    )
    .await?;

    let guild_id = conn.last_insert_id().unwrap();

    conn.exec_drop(
        include_str!("../../../sql/create_guild_ranks.sql"),
        params! {
            "guild_id" => guild_id,
            "leader_rank_name" => SETTINGS.guild.default_leader_rank_name.clone(),
            "recruiter_rank_name" => SETTINGS.guild.default_recruiter_rank_name.clone(),
            "new_member_rank_name" => SETTINGS.guild.default_new_member_rank_name.clone(),
        },
    )
    .await?;

    Ok(())
}
