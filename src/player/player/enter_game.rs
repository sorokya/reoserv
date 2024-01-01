use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            WelcomeCode, WelcomeReplyServerPacket, WelcomeReplyServerPacketWelcomeCodeData,
            WelcomeReplyServerPacketWelcomeCodeDataEnterGame,
        },
        PacketAction, PacketFamily,
    },
};
use std::{io::Cursor, path::Path};
use tokio::io::{AsyncBufReadExt, AsyncReadExt};

use crate::{errors::WrongSessionIdError, player::ClientState};

use super::Player;

impl Player {
    pub async fn enter_game(&mut self, session_id: i32) -> bool {
        let actual_session_id = match self.take_session_id() {
            Ok(session_id) => session_id,
            Err(e) => {
                self.close(format!("Error getting session id: {}", e)).await;
                return false;
            }
        };

        if actual_session_id != session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(actual_session_id, session_id)
            ))
            .await;
            return false;
        }

        let mut character = match self.character.take() {
            Some(character) => character,
            None => {
                self.close("Player attempted to enter game with no character selected".to_string())
                    .await;
                return false;
            }
        };

        let map = match self.world.get_map(character.map_id).await {
            Ok(map) => map,
            Err(e) => {
                self.close(format!("Error getting map: {}", e)).await;
                return false;
            }
        };

        self.map = Some(map.clone());
        self.state = ClientState::InGame;

        let items = character.items.clone();
        let spells = character.spells.clone();
        let weight = character.get_weight();

        if let Some(relog_coords) = map.get_relog_coords().await {
            character.coords = relog_coords;
        }

        map.enter(Box::new(character), None).await;

        let nearby_info = map.get_nearby_info(self.id).await;

        let reply = WelcomeReplyServerPacket {
            welcome_code: WelcomeCode::EnterGame,
            welcome_code_data: Some(WelcomeReplyServerPacketWelcomeCodeData::EnterGame(
                WelcomeReplyServerPacketWelcomeCodeDataEnterGame {
                    news: get_news().await,
                    weight,
                    items,
                    spells,
                    nearby: nearby_info,
                },
            )),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize WelcomeReplyServerPacket: {}", e);
            return false;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Reply,
                PacketFamily::Welcome,
                writer.to_byte_array(),
            )
            .await;
        true
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
