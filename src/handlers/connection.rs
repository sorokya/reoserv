use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::{client::connection::Accept, PacketAction},
};

use crate::player::PlayerHandle;

async fn accept(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Accept::default();
    packet.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            player.close(format!("Failed to get player id: {}", e));
            return;
        }
    };

    if player_id != packet.player_id {
        player.close(format!(
            "sending invalid connection id: Got {}, expected {}.",
            packet.player_id, player_id
        ));
    }

    let expected_multiples = match player.get_encoding_multiples().await {
        Ok(multiples) => multiples,
        Err(e) => {
            player.close(format!("Failed to get encoding multiples: {}", e));
            return;
        }
    };

    if expected_multiples[0] as EOShort != packet.encode_multiple
        || expected_multiples[1] as EOShort != packet.decode_multiple
    {
        player.close(format!(
            "sending invalid encoding multiples: Got {:?}, expected {:?}.",
            [packet.decode_multiple, packet.encode_multiple],
            expected_multiples
        ));
    }
}

pub async fn connection(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Accept => accept(reader, player).await,
        PacketAction::Ping => player.pong(),
        _ => error!("Unhandled packet Connection_{:?}", action),
    }
}
