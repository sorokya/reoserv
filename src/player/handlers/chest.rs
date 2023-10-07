use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::chest::{Open, Take},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn open(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Open::default();
    packet.deserialize(&reader);

    let player_id = player.get_player_id().await;
    if let Err(e) = player_id {
        error!("Error getting player id {}", e);
        return;
    }

    let player_id = player_id.unwrap();

    let map = player.get_map().await;
    if let Err(e) = map {
        error!("Error getting map {}", e);
        return;
    }

    let map = map.unwrap();

    map.open_chest(player_id, packet.coords);
}

async fn take(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Take::default();
    packet.deserialize(&reader);

    let player_id = player.get_player_id().await;
    if let Err(e) = player_id {
        error!("Error getting player id {}", e);
        return;
    }

    let player_id = player_id.unwrap();

    let map = player.get_map().await;
    if let Err(e) = map {
        error!("Error getting map {}", e);
        return;
    }

    let map = map.unwrap();

    map.take_chest_item(player_id, packet.coords, packet.take_item_id);
}

pub async fn chest(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Open => open(reader, player).await,
        PacketAction::Take => take(reader, player).await,
        _ => error!("Unhandled packet Chest_{:?}", action),
    }
}
