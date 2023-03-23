use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::client::connection::Accept,
};

use crate::{player::PlayerHandle, Bytes};

pub async fn accept(buf: Bytes, player: PlayerHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Accept::default();
    let reader = StreamReader::new(buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let player_id = player.get_player_id().await?;
    if player_id != packet.player_id {
        player.close(format!(
            "sending invalid connection id: Got {}, expected {}.",
            packet.player_id, player_id
        ));
    }

    let mut expected_multiples = player.get_encoding_multiples().await?;
    expected_multiples.reverse();
    if expected_multiples[0] as EOShort != packet.decode_multiple
        || expected_multiples[1] as EOShort != packet.encode_multiple
    {
        player.close(format!(
            "sending invalid encoding multiples: Got {:?}, expected {:?}.",
            [packet.decode_multiple, packet.encode_multiple],
            expected_multiples
        ));
    }

    Ok(())
}
