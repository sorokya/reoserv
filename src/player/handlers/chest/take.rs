pub use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::chest::Take,
};

use crate::player::PlayerHandle;

pub async fn take(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = Take::default();
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

    map.take_chest_item(player_id, packet.coords, packet.take_item_id);
}
