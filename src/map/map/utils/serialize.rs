use bytes::Bytes;
use eolib::data::{EoSerialize, EoWriter};
use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub fn serialize(&self, respond_to: oneshot::Sender<Bytes>) {
        let mut writer = EoWriter::with_capacity(self.file_size as usize);
        self.file.serialize(&mut writer);
        let _ = respond_to.send(writer.to_byte_array());
    }
}
