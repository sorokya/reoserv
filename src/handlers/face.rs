use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::FacePlayerClientPacket, PacketAction},
};

use crate::player::PlayerHandle;

async fn player(reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    let packet = match FacePlayerClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing FacePlayerClientPacket {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.face(player_id, packet.direction);
    }
}

pub async fn face(action: PacketAction, reader: EoReader, player_handle: PlayerHandle) {
    match action {
        PacketAction::Player => player(reader, player_handle).await,
        _ => error!("Unhandled packet Face_{:?}", action),
    }
}
