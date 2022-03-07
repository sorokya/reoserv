use eo::data::{EOShort, map::MapFile, Serializeable};
use tokio::sync::mpsc::UnboundedReceiver;

use super::Command;

pub struct Map {
    pub rx: UnboundedReceiver<Command>,
    id: EOShort,
    file: MapFile,
}

impl Map {
    pub fn new(id: EOShort, file: MapFile, rx: UnboundedReceiver<Command>) -> Self {
        Self { id, file, rx }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::GetHashAndSize { respond_to } => {
                let _ = respond_to.send((self.file.hash, self.file.size));
            },
            Command::Serialize { respond_to } => {
                let _ = respond_to.send(self.file.serialize());
            },
        }
    }
}