use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::attack::Use, PacketAction},
};

use crate::player::PlayerHandle;

async fn r#use(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Use::default();
    packet.deserialize(&reader);

    // TODO: implement anti speed

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

    map.attack(player_id, packet.direction);
}

pub async fn attack(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Use => r#use(reader, player).await,
        _ => error!("Unhandled packet Attack_{:?}", action),
    }
}
