use bytes::Bytes;
use eolib::data::{EoSerialize, EoWriter};
use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub fn serialize(&self, respond_to: oneshot::Sender<Bytes>) {
        let mut writer = EoWriter::with_capacity(self.file_size as usize);

        if let Err(e) = self.file.serialize(&mut writer) {
            error!("Failed to serialize map file: {}", e);
            return;
        }

        let _ = respond_to.send(writer.to_byte_array());
    }
}
