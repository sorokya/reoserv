use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::face::Player, PacketAction},
};

use crate::player::PlayerHandle;

async fn player(reader: StreamReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    let mut packet = Player::default();
    packet.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        map.face(player_id, packet.direction);
    }
}

pub async fn face(action: PacketAction, reader: StreamReader, player_handle: PlayerHandle) {
    match action {
        PacketAction::Player => player(reader, player_handle).await,
        _ => error!("Unhandled packet Face_{:?}", action),
    }
}
