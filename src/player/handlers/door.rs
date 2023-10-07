use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::door::Open, PacketAction},
};

use crate::player::PlayerHandle;

async fn open(reader: StreamReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let mut open = Open::default();
    open.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        map.open_door(player_id, open.coords);
    }
}

pub async fn door(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Open => open(reader, player).await,
        _ => error!("Unhandled packet Door_{:?}", action),
    }
}
