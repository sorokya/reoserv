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
    EnsureValidSequenceForAccountCreation {
        respond_to: oneshot::Sender<()>,
    },
    GetSequenceStart {
        respond_to: oneshot::Sender<EOInt>,
    },
    GetSequenceBytes {
        respond_to: oneshot::Sender<(EOShort, EOChar)>,
    },
    GetEncodeMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
    },
    GetIpAddr {
        respond_to: oneshot::Sender<String>,
    },
    SetState(State),
    SetBusy(bool),
    Ping,
    Pong,
}
