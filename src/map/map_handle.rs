use eo::{
    data::{map::MapFile, EOByte, EOInt, EOShort, EOChar},
    net::{packets::server::map_info, NearbyInfo},
    world::{Direction, WarpAnimation},
};
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::{PacketBuf, character::Character};

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

    // pub fn drop_player(&self, player_id: EOShort, coords: Coords) {
    //     let _ = self.tx.send(Command::DropPlayer(player_id, coords));
    // }

    pub async fn enter(&self, character: Character) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Enter(character, tx));
        rx.await.unwrap();
    }

    pub fn face(&self, player_id: EOShort, direction: Direction) {
        let _ = self.tx.send(Command::Face(player_id, direction));
    }

    pub async fn get_map_info(
        &self,
        player_ids: Option<Vec<EOShort>>,
        npc_indexes: Option<Vec<EOChar>>,
    ) -> Result<map_info::Reply, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMapInfo {
            player_ids,
            _npc_indexes: npc_indexes,
            respond_to: tx,
        });
        rx.await?
    }

    pub async fn get_hash_and_size(&self) -> ([EOByte; 4], EOInt) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetHashAndSize { respond_to: tx });
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

    pub async fn leave(&self, target_player_id: EOShort) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Leave {
            target_player_id,
            warp_animation: None,
            respond_to: tx,
        });
        let _ = rx.await;
    }

    pub async fn _leave_animated(&self, target_player_id: EOShort, warp_animation: WarpAnimation) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Leave {
            target_player_id,
            warp_animation: Some(warp_animation),
            respond_to: tx,
        });
        let _ = rx.await;
    }

    pub async fn serialize(&self) -> PacketBuf {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Serialize { respond_to: tx });
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
