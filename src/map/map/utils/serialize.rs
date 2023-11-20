use bytes::Bytes;
use eo::data::{Serializeable, StreamBuilder};
use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub fn serialize(&self, respond_to: oneshot::Sender<Bytes>) {
        let mut builder = StreamBuilder::with_capacity(self.file_size as usize);
        self.file.serialize(&mut builder);
        let _ = respond_to.send(builder.get());
    }
}
