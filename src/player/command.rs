use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, Family},
};
use tokio::sync::oneshot;

use crate::PacketBuf;

use super::{State, InvalidStateError};

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
    GetEncodingMultiples {
        respond_to: oneshot::Sender<[EOByte; 2]>,
    },
    GetIpAddr {
        respond_to: oneshot::Sender<String>,
    },
    GetAccountId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    GetPlayerId {
        respond_to: oneshot::Sender<EOShort>,
    },
    GetCharacterId {
        respond_to: oneshot::Sender<Result<EOInt, InvalidStateError>>,
    },
    SetState(State),
    SetBusy(bool),
    Ping,
    Pong,
}
