use std::{path::Path, io::Cursor};

use eo::{
    data::{EOShort, EOChar},
    protocol::{server::welcome::{Reply, ReplyData, ReplyEnterGame}, Weight, WelcomeReply},
};
use tokio::{sync::oneshot, io::{AsyncReadExt, AsyncBufReadExt}};

use crate::{errors::{WrongSessionIdError, DataNotFoundError}, player::{PlayerHandle, ClientState}, map::MapHandle};

use super::World;

impl World {
    pub async fn enter_game(
        &mut self,
        player: PlayerHandle,
        session_id: EOShort,
        respond_to: oneshot::Sender<Result<Reply, Box<dyn std::error::Error + Send + Sync>>>,
    ) {
        let actual_session_id = player.take_session_id().await;
        if let Err(e) = actual_session_id {
            let _ = respond_to.send(Err(Box::new(e)));
            return;
        }

        let actual_session_id = actual_session_id.unwrap();
        if actual_session_id != session_id {
            let _ = respond_to.send(Err(Box::new(WrongSessionIdError::new(
                actual_session_id,
                session_id,
            ))));
            return;
        }
        
        let map_id = match player.get_map_id().await {
            Ok(map_id) => map_id,
            Err(e) => {
                let _ = respond_to.send(Err(Box::new(e)));
                return;
            }
        };

        if let Some(maps) = self.maps.as_ref() {
            if let Some(map) = maps.get(&map_id) {
                let player = player.to_owned();
                let map = map.to_owned();
                let _ = tokio::task::Builder::new()
                    .name("enter_game")
                    .spawn(async move {
                        let result = enter_game(map, player).await;
                        let _ = respond_to.send(result);
                    });
            } else {
                // TODO: Move character to safe map
                let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                    "Map".to_string(),
                    map_id,
                ))));
            }
        } else {
            let _ = respond_to.send(Err(Box::new(DataNotFoundError::new(
                "Map".to_string(),
                map_id,
            ))));
        }
    }
}

pub async fn enter_game(
    map: MapHandle,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    let player_id = player.get_player_id().await;
    player.set_map(map.clone());
    player.set_state(ClientState::Playing);
    let character = player.take_character().await?;

    let weight = Weight {
        current: character.weight as EOChar,
        max: character.max_weight as EOChar,
    };
    let items = character.items.clone();
    let spells = character.spells.clone();

    map.enter(character, None).await;
    let nearby_info = map.get_nearby_info(player_id).await;
    Ok(Reply {
        reply_code: WelcomeReply::EnterGame,
        data: ReplyData::EnterGame(ReplyEnterGame {
            news: get_news().await,
            weight,
            items,
            spells,
            nearby: nearby_info,
        }),
    })
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
