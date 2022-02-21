use eo::{
    data::{EOByte, EOShort, Serializeable, StreamReader},
    net::packets::client::connection::Accept,
};

use crate::{player::Command, PacketBuf, Tx};

pub async fn accept(
    buf: PacketBuf,
    player_id: EOShort,
    decode_multiple: EOByte,
    encode_multiple: EOByte,
    tx: &Tx,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut packet = Accept::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    if player_id != packet.player_id {
        tx.send(Command::Close(format!(
            "invalid player id. Got {}, expected {}.",
            packet.player_id, player_id
        )))?;
    }

    let expected_multiples = [decode_multiple, encode_multiple];

    if expected_multiples != packet.encoding_multiples {
        tx.send(Command::Close(format!(
            "invalid encode multiples. Got {:?}, expected {:?}.",
            packet.encoding_multiples, expected_multiples
        )))?;
    }

    Ok(())
}
