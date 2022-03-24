use eo::{
    character::Emote,
    data::{map::MapFile, EOChar, EOInt, EOShort, EOThree},
    net::{packets::server::map_info, NearbyInfo},
    world::{Direction, TinyCoords, WarpAnimation},
};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::{character::Character, PacketBuf};

use super::{Command, Map};

#[derive(Debug, Clone)]
pub struct MapHandle {
    tx: UnboundedSender<Command>,
}

impl MapHandle {
    pub fn new(id: EOShort, file: MapFile) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let map = Map::new(file, rx);
        tokio::task::Builder::new()
            .name(&format!("Map {}", id))
            .spawn(run_map(map));

        Self { tx }
    }

    pub fn emote(&self, target_player_id: u16, emote: Emote) {
        let _ = self.tx.send(Command::Emote {
            target_player_id,
            emote,
        });
    }

    pub async fn enter(&self, character: Box<Character>, warp_animation: Option<WarpAnimation>) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Enter {
            character,
            warp_animation,
            respond_to: tx,
        });
        rx.await.unwrap();
    }

    pub fn face(&self, target_player_id: EOShort, direction: Direction) {
        let _ = self.tx.send(Command::Face {
            target_player_id,
            direction,
        });
    }

    pub async fn get_character(&self, player_id: EOShort) -> Option<Box<Character>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacter {
            player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_dimensions(&self) -> (EOChar, EOChar) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetDimensions { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_map_info(
        &self,
        player_ids: Option<Vec<EOShort>>,
        npc_indexes: Option<Vec<EOChar>>,
    ) -> map_info::Reply {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMapInfo {
            player_ids,
            npc_indexes,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_nearby_info(&self, target_player_id: EOShort) -> NearbyInfo {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetNearbyInfo {
            target_player_id,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub async fn get_rid_and_size(&self) -> ([EOShort; 2], EOInt) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetRidAndSize { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn leave(
        &self,
        target_player_id: EOShort,
        warp_animation: Option<WarpAnimation>,
    ) -> Character {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Leave {
            target_player_id,
            warp_animation,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub fn open_door(&self, target_player_id: EOShort, door_coords: TinyCoords) {
        let _ = self.tx.send(Command::OpenDoor {
            target_player_id,
            door_coords,
        });
    }

    pub fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        let _ = self.tx.send(Command::SendChatMessage {
            target_player_id,
            message,
        });
    }

    pub async fn serialize(&self) -> PacketBuf {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Serialize { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn walk(
        &self,
        target_player_id: EOShort,
        timestamp: EOThree,
        coords: TinyCoords,
        direction: Direction,
    ) {
        let _ = self.tx.send(Command::Walk {
            target_player_id,
            timestamp,
            coords,
            direction,
        });
    }
}

async fn run_map(mut map: Map) {
    loop {
        if let Some(command) = map.rx.recv().await {
            map.handle_command(command).await;
        }
    }
}
