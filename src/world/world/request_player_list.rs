use eolib::protocol::net::{
    server::{
        InitInitServerPacket, InitInitServerPacketReplyCodeData,
        InitInitServerPacketReplyCodeDataPlayersList, InitReply, OnlinePlayer, PlayersList,
    },
    PacketAction, PacketFamily,
};

use crate::utils::pad_string;

use super::World;

impl World {
    pub fn request_player_list(&self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.clone(),
            None => return,
        };

        let online_players = self.players.clone();
        let parties = self.parties.clone();

        tokio::spawn(async move {
            let mut players = Vec::with_capacity(online_players.len());

            for (online_player_id, online_player) in online_players {
                let character = match online_player.get_character().await {
                    Ok(character) => character,
                    Err(_) => {
                        continue;
                    }
                };

                let in_party = parties
                    .iter()
                    .any(|p| p.leader == online_player_id || p.members.contains(&online_player_id));

                players.push(OnlinePlayer {
                    name: character.name.clone(),
                    title: character.title.clone().unwrap_or_default(),
                    level: character.level,
                    icon: character.get_icon(in_party),
                    class_id: character.class,
                    guild_tag: pad_string(&character.guild_tag.clone().unwrap_or_default(), 3),
                });
            }

            player.send(
                PacketAction::Init,
                PacketFamily::Init,
                &InitInitServerPacket {
                    reply_code: InitReply::PlayersList,
                    reply_code_data: Some(InitInitServerPacketReplyCodeData::PlayersList(
                        InitInitServerPacketReplyCodeDataPlayersList {
                            players_list: PlayersList { players },
                        },
                    )),
                },
            );
        });
    }
}
