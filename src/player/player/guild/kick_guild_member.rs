use super::super::Player;
use eolib::{protocol::net::server::GuildReply};

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

        if !character.is_guild_leader() {
            return;
        }

        match self.world.get_character_by_name(member_name.clone()).await {
            Ok(member) => {
                if member.guild_tag != character.guild_tag {
                    send_reply!(self, GuildReply::RemoveNotMember);
                    return;
                }

                if member.is_guild_leader() {
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
                self.send_server_message("Offline kicking not currently supported")
                    .await;
            }
        }
    }
}
