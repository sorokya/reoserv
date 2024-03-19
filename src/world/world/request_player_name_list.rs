use eolib::protocol::net::{
    server::{
        InitInitServerPacket, InitInitServerPacketReplyCodeData,
        InitInitServerPacketReplyCodeDataPlayersListFriends, InitReply, PlayersListFriends,
    },
    PacketAction, PacketFamily,
};

use super::World;

impl World {
    pub fn request_player_name_list(&self, player_id: i32) {
        let player = match self.players.get(&player_id) {
            Some(player) => player.clone(),
            None => return,
        };

        let online_players = self.players.clone();

        tokio::spawn(async move {
            let mut players = Vec::with_capacity(online_players.len());

            for (_, online_player) in online_players {
                let character = match online_player.get_character().await {
                    Ok(character) => character,
                    Err(e) => {
                        error!("Error getting character: {}", e);
                        continue;
                    }
                };

                players.push(character.name.clone());
            }

            let packet = InitInitServerPacket {
                reply_code: InitReply::PlayersListFriends,
                reply_code_data: Some(InitInitServerPacketReplyCodeData::PlayersListFriends(
                    InitInitServerPacketReplyCodeDataPlayersListFriends {
                        players_list: PlayersListFriends { players },
                    },
                )),
            };

            player.send(PacketAction::Init, PacketFamily::Init, &packet);
        });
    }
}
