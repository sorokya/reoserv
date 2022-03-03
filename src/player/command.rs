use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, Family},
};
use tokio::sync::oneshot;

use crate::PacketBuf;

use super::State;

#[derive(Debug)]
pub enum Command {
    Send(Action, Family, PacketBuf),
    PongNewSequence {
        respond_to: oneshot::Sender<()>,
    },
    GenSequence {
        respond_to: oneshot::Sender<EOInt>,
    },
    Close(String),
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetEncodeMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
    },
    SetState(State),
    SetBusy(bool),
}
