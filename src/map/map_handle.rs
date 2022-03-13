use eo::{
    data::{map::MapFile, EOByte, EOChar, EOInt, EOShort, EOThree},
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

    pub fn open_door(&self, target_player_id: EOShort, door_coords: TinyCoords) {
        let _ = self.tx.send(Command::OpenDoor {
            target_player_id,
            door_coords,
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
