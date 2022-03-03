use eo::{
    data::{EOShort, Serializeable, StreamReader},
    net::packets::client::connection::Accept,
};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{player::Command, PacketBuf};

pub async fn accept(
    buf: PacketBuf,
    player_id: EOShort,
    player: UnboundedSender<Command>,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Accept::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    if player_id != packet.player_id {
        player.send(Command::Close(format!(
            "invalid player id. Got {}, expected {}.",
            packet.player_id, player_id
        )))?;
    }

    let (tx, rx) = oneshot::channel();
    player.send(Command::GetEncodeMultiples { respond_to: tx })?;
    let mut expected_multiples = rx.await.unwrap();
    expected_multiples.reverse();

    if expected_multiples != packet.encoding_multiples {
        player.send(Command::Close(format!(
            "invalid encode multiples. Got {:?}, expected {:?}.",
            packet.encoding_multiples, expected_multiples
        )))?;
    }

    Ok(())
}
