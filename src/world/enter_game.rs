use std::{io::Cursor, path::Path};

use eo::net::{
    packets::server::welcome::{EnterGame, Reply},
    replies::WelcomeReply,
};

use crate::{map::MapHandle, player::PlayerHandle};

use tokio::io::{AsyncBufReadExt, AsyncReadExt};

pub async fn enter_game(
    map: MapHandle,
    player: PlayerHandle,
) -> Result<Reply, Box<dyn std::error::Error + Send + Sync>> {
    let player_id = player.get_player_id().await;
    player.set_map(map.clone());
    map.enter(player_id, player.clone());
    let _ = player.calculate_stats().await;
    let weight = player.get_weight().await?;
    let items = player.get_items().await?;
    let spells = player.get_spells().await?;
    let nearby_info = map.get_nearby_info(player_id).await;
    Ok(Reply {
        reply: WelcomeReply::EnterGame,
        select_character: None,
        enter_game: Some(EnterGame {
            news: get_news().await,
            weight,
            items,
            spells,
            nearby_info,
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
            for i in 0..9 {
                match lines.next_line().await {
                    Ok(line) => match line {
                        Some(line) => news[i] = line,
                        None => {}
                    },
                    Err(_) => {}
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
