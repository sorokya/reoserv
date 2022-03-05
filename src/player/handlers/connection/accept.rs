use eo::{
    data::{EOShort, Serializeable, StreamReader},
    net::packets::client::connection::Accept,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn accept(buf: PacketBuf, player_id: EOShort, player: PlayerHandle) {
    let mut packet = Accept::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    if player_id != packet.player_id {
        player.close(format!(
            "sending invalid player id: Got {}, expected {}.",
            packet.player_id, player_id
        ));
    }

    let mut expected_multiples = player.get_encoding_multiples().await;
    expected_multiples.reverse();
    if expected_multiples != packet.encoding_multiples {
        player.close(format!(
            "sending invalid encoding multiples: Got {:?}, expected {:?}.",
            packet.encoding_multiples, expected_multiples
        ));
    }
}
