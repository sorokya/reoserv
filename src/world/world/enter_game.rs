use std::{io::Cursor, path::Path};

use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{
        server::welcome::{Reply, ReplyData, ReplyEnterGame},
        PacketAction, PacketFamily, WelcomeReply,
    },
};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::{
    errors::{DataNotFoundError, WrongSessionIdError},
    player::ClientState,
};

use super::World;

impl World {
    pub async fn enter_game(&mut self, player_id: EOShort, session_id: EOShort) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let actual_session_id = player.take_session_id().await;
        if let Err(e) = actual_session_id {
            player.close(format!("Error getting session id: {}", e));
            return;
        }

        let actual_session_id = actual_session_id.unwrap();
        if actual_session_id != session_id {
            player.close(format!(
                "{}",
                WrongSessionIdError::new(actual_session_id, session_id)
            ));
            return;
        }

        let map_id = match player.get_map_id().await {
            Ok(map_id) => map_id,
            Err(e) => {
                player.close(format!("Error getting map id: {}", e));
                return;
            }
        };

        if let Some(maps) = self.maps.as_ref() {
            if let Some(map) = maps.get(&map_id) {
                let player = player.to_owned();
                let map = map.to_owned();

                let player_id = player.get_player_id().await;

                if player_id.is_err() {
                    return;
                }

                let player_id = player_id.unwrap();

                player.set_map(map.clone());
                player.set_state(ClientState::Playing);
                let character = player.take_character().await;

                if let Err(e) = character {
                    player.close(format!("Error getting character from player: {:?}", e));
                    return;
                }

                let mut character = character.unwrap();
                let items = character.items.clone();
                let spells = character.spells.clone();
                let weight = character.get_weight();

                if let Some(relog_coords) = map.get_relog_coords().await {
                    character.coords = relog_coords;
                }

                map.enter(character, None).await;
                let nearby_info = map.get_nearby_info(player_id).await;
                let reply = Reply {
                    reply_code: WelcomeReply::EnterGame,
                    data: ReplyData::EnterGame(ReplyEnterGame {
                        news: get_news().await,
                        weight,
                        items,
                        spells,
                        nearby: nearby_info,
                    }),
                };

                let mut builder = StreamBuilder::new();
                reply.serialize(&mut builder);
                player.send(PacketAction::Reply, PacketFamily::Welcome, builder.get());
            } else {
                player.close(format!(
                    "{}",
                    DataNotFoundError::new("Map".to_string(), map_id,)
                ));
            }
        } else {
            player.close(format!(
                "{}",
                DataNotFoundError::new("Map".to_string(), map_id,)
            ));
        }
    }
}

async fn get_news() -> [String; 9] {
    match tokio::fs::File::open(Path::new("news.txt")).await {
        Ok(mut file) => {
            let mut buf: Vec<u8> = Vec::new();
            file.read_to_end(&mut buf).await.unwrap();
            let cursor = Cursor::new(buf);
            let mut lines = cursor.lines();
            let mut news = [
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
            ];
            for entry in &mut news {
                if let Ok(Some(line)) = lines.next_line().await {
                    *entry = line
                }
            }
            news
        }
        Err(e) => {
            error!("Failed to open news.txt: {}", e);
            [
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
                String::default(),
            ]
        }
    }
}
