use eo::{
    data::{map::MapFile, EOByte, EOInt, EOShort},
    net::NearbyInfo,
};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::{player::PlayerHandle, PacketBuf};

use super::{Command, Map};

#[derive(Debug)]
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

    pub fn enter(&self, player_id: EOShort, player: PlayerHandle) {
        let _ = self.tx.send(Command::Enter(player_id, player));
    }

    pub async fn get_hash_and_size(&self) -> ([EOByte; 4], EOInt) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetHashAndSize { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn serialize(&self) -> PacketBuf {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Serialize { respond_to: tx });
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
}

async fn run_map(mut map: Map) {
    loop {
        if let Some(command) = map.rx.recv().await {
            map.handle_command(command).await;
        }
    }
}
