use crate::{
    player::{Command, State},
    PacketBuf,
};
use eo::{
    data::{EOByte, EOShort, Serializeable, StreamReader},
    net::{
        packets::{client::init::Init, server},
        replies::InitReply,
        stupid_hash, Action, Family,
    },
};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

pub async fn init(
    buf: PacketBuf,
    player_id: EOShort,
    player: UnboundedSender<Command>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Init::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let mut reply = server::init::Init::new();
    reply.reply_code = InitReply::OK;

    let mut init_ok = server::init::InitOk::new();
    init_ok.challenge_response = stupid_hash(packet.challenge);
    init_ok.player_id = player_id;

    let (tx, rx) = oneshot::channel();
    player.send(Command::GetSequenceBytes { respond_to: tx })?;
    let sequence_bytes = rx.await.unwrap();
    init_ok.sequence_bytes = [sequence_bytes.0 as EOByte, sequence_bytes.1];

    let (tx, rx) = oneshot::channel();
    player.send(Command::GetEncodeMultiples { respond_to: tx })?;
    init_ok.encoding_multiples = rx.await.unwrap();

    debug!("Reply code: {:?}, data: {:?}", reply.reply_code, init_ok);

    reply.reply = Box::new(init_ok);

    player.send(Command::SetState(State::Initialized))?;
    player.send(Command::Send(Action::Init, Family::Init, reply.serialize()))?;

    Ok(())
}
