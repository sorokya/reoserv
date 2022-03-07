use eo::data::{EOShort, map::MapFile, EOByte, EOInt};
use tokio::sync::{mpsc::{UnboundedSender, self}, oneshot};

use crate::PacketBuf;

use super::{Command, Map};

#[derive(Debug)]
pub struct MapHandle {
    tx: UnboundedSender<Command>,
}

impl MapHandle {
    pub fn new(id: EOShort, file: MapFile) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let map = Map::new(id, file, rx);
        tokio::task::Builder::new()
            .name("run_map")
            .spawn(run_map(map));

        Self { tx }
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
}

async fn run_map(mut map: Map) {
    loop {
        if let Some(command) = map.rx.recv().await {
            map.handle_command(command).await;
        }
    }
}
