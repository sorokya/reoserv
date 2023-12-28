use eolib::{data::{EoReader, EoSerialize}, protocol::net::{PacketAction, client::ConnectionAcceptClientPacket}};

use crate::player::PlayerHandle;

fn accept(reader: EoReader, player: PlayerHandle) {
    let accept = match ConnectionAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            player.close(format!("Failed to deserialize ConnectionAcceptClientPacket: {}", e));
            return;
        }
    };

    player.complete_handshake(accept.player_id, accept.client_encryption_multiple, accept.server_encryption_multiple);

    // if player_id != accept.player_id {
    //     player.close(format!(
    //         "sending invalid connection id: Got {}, expected {}.",
    //         accept.player_id, player_id
    //     ));
    // }

    // let expected_multiples = match player.get_encoding_multiples().await {
    //     Ok(multiples) => multiples,
    //     Err(e) => {
    //         player.close(format!("Failed to get encoding multiples: {}", e));
    //         return;
    //     }
    // };

    // if expected_multiples[0] as i32 != accept.client_encryption_multiple
    //     || expected_multiples[1] as i32 != accept.server_encryption_multiple
    // {
    //     player.close(format!(
    //         "sending invalid encoding multiples: Got {:?}, expected {:?}.",
    //         [accept.client_encryption_multiple, accept.server_encryption_multiple],
    //         expected_multiples
    //     ));
    // }
}

pub async fn connection(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Accept => accept(reader, player),
        PacketAction::Ping => player.pong(),
        _ => error!("Unhandled packet Connection_{:?}", action),
    }
}
