use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::item,
};

use crate::player::PlayerHandle;

pub async fn drop(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = item::Drop::default();
    packet.deserialize(&reader);

    debug!("{:?}", packet);

    let player_id = player.get_player_id().await;

    if let Err(e) = player_id {
        error!("Failed to get player id: {}", e);
        return;
    }

    let player_id = player_id.unwrap();

    let map = player.get_map().await;

    if let Err(e) = map {
        error!("Failed to get map: {}", e);
        return;
    }

    map.unwrap().drop_item(player_id, packet.drop_item, packet.coords);
}
