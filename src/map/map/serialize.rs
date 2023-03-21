use eo::data::{EOByte, Serializeable};
use tokio::sync::oneshot;

use super::Map;

impl Map {
    pub fn serialize(&self, respond_to: oneshot::Sender<Vec<EOByte>>) {
        let _ = respond_to.send(self.file.serialize());
    }
}
