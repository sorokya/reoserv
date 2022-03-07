use eo::data::{EOByte, EOInt};
use tokio::sync::oneshot;

use crate::PacketBuf;

#[derive(Debug)]
pub enum Command {
    GetHashAndSize {
        respond_to: oneshot::Sender<([EOByte; 4], EOInt)>,
    },
    Serialize {
        respond_to: oneshot::Sender<PacketBuf>,
    },
}
