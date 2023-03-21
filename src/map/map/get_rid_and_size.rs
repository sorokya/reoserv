use eo::data::{EOInt, EOShort};
use tokio::sync::oneshot;

use super::Map;

impl Map {
    pub fn get_rid_and_size(&self, respond_to: oneshot::Sender<([EOShort; 2], EOInt)>) {
        let _ = respond_to.send((self.file.rid, self.file_size));
    }
}
