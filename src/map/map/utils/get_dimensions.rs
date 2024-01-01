use tokio::sync::oneshot;

use super::super::Map;

impl Map {
    pub fn get_dimensions(&self, respond_to: oneshot::Sender<(i32, i32)>) {
        let _ = respond_to.send((self.file.width, self.file.height));
    }
}
