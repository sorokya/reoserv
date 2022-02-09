use crate::{player::Command, PacketBuf, Tx};
use eo::{
    data::{EOByte, EOChar, EOShort, Serializeable, StreamReader},
    net::{
        packets::{client::init::Init, server},
        replies::InitReply, stupid_hash, Action, Family,
    },
};

pub async fn init(
    buf: PacketBuf,
    player_id: EOShort,
    sequence_bytes: (EOShort, EOChar),
    decode_multiple: EOByte,
    encode_multiple: EOByte,
    tx: &Tx,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut packet = Init::default();
    let reader = StreamReader::new(&buf);
    reader.seek(2);
    packet.deserialize(&reader);

    let mut reply = server::init::Init::new();
    reply.reply_code = InitReply::OK;

    let mut init_ok = server::init::InitOk::new();
    init_ok.challenge_response = stupid_hash(packet.challenge);
    init_ok.player_id = player_id;

    init_ok.sequence_bytes = [sequence_bytes.0 as EOByte, sequence_bytes.1];
    init_ok.encoding_multiples = [
     decode_multiple,
        encode_multiple,
    ];

    reply.reply = Box::new(init_ok);

    tx.send(Command::Send(
        Action::Init,
        Family::Init,
        reply.serialize(),
    ))?;

    Ok(())
}
