use eo::data::EOChar;
use tokio::sync::oneshot;

use super::Map;

impl Map {
    pub fn get_dimensions(&self, respond_to: oneshot::Sender<(EOChar, EOChar)>) {
        let _ = respond_to.send((self.file.width, self.file.height));
    }
}
