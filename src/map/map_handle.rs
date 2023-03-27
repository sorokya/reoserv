use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort},
    protocol::{server::range, Coords, Direction, Emote, NearbyInfo, WarpAnimation, ShortItem},
    pubs::EmfFile,
};
use mysql_async::Pool;
use tokio::sync::{
    mpsc::{self, UnboundedSender},
    oneshot,
};

use crate::{character::Character};

use super::{Command, Map};

#[derive(Debug, Clone)]
pub struct MapHandle {
    tx: UnboundedSender<Command>,
}

impl MapHandle {
    pub fn new(id: EOShort, file_size: EOInt, pool: Pool, file: EmfFile) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let map = Map::new(file_size, file, pool, rx);
        let _ = tokio::task::Builder::new()
            .name(&format!("Map {}", id))
            .spawn(run_map(map));

        Self { tx }
    }

    pub fn drop_item(&self, target_player_id: EOShort, item: ShortItem, coords: Coords) {
        let _ = self.tx.send(Command::DropItem {
            target_player_id,
            item,
            coords,
        });
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

    // TODO: use coords!
    pub async fn get_dimensions(&self) -> (EOChar, EOChar) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetDimensions { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn get_item(&self, target_player_id: EOShort, item_index: EOShort) {
        let _ = self.tx.send(Command::GetItem { item_index, target_player_id });
    }

    pub async fn get_map_info(
        &self,
        player_ids: Vec<EOShort>,
        npc_indexes: Vec<EOChar>,
    ) -> range::Reply {
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

    pub fn give_item(&self, target_player_id: EOShort, item_id: EOShort, amount: EOInt) {
        let _ = self.tx.send(Command::GiveItem {
            target_player_id,
            item_id,
            amount,
        });
    }

    pub fn junk_item(&self, target_player_id: EOShort, item_id: EOShort, amount: EOInt) {
        let _ = self.tx.send(Command::JunkItem {
            target_player_id,
            item_id,
            amount,
        });
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

    pub fn open_door(&self, target_player_id: EOShort, door_coords: Coords) {
        let _ = self.tx.send(Command::OpenDoor {
            target_player_id,
            door_coords,
        });
    }

    pub fn request_paperdoll(&self, player_id: EOShort, target_player_id: EOShort) {
        let _ = self.tx.send(Command::RequestPaperdoll {
            player_id,
            target_player_id,
        });
    }

    pub async fn save(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Save { respond_to: tx });
        rx.await.unwrap();
    }

    pub fn send_chat_message(&self, target_player_id: EOShort, message: String) {
        let _ = self.tx.send(Command::SendChatMessage {
            target_player_id,
            message,
        });
    }

    pub async fn serialize(&self) -> Bytes {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::Serialize { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn spawn_npcs(&self) {
        let _ = self.tx.send(Command::SpawnNpcs);
    }

    pub fn act_npcs(&self) {
        let _ = self.tx.send(Command::ActNpcs);
    }

    pub fn walk(
        &self,
        target_player_id: EOShort,
        direction: Direction,
    ) {
        let _ = self.tx.send(Command::Walk {
            target_player_id,
            direction,
        });
    }

    pub fn attack(&self, target_player_id: EOShort, direction: Direction) {
        let _ = self.tx.send(Command::Attack {
            target_player_id,
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
