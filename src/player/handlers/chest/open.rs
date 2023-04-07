use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::chest::Open,
};

use crate::player::PlayerHandle;

pub async fn open(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = Open::default();
    packet.deserialize(&reader);

    debug!("{:?}", packet);

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
