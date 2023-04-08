use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::item,
};

use crate::player::PlayerHandle;

pub async fn junk(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = item::Junk::default();
    packet.deserialize(&reader);

    debug!("{:?}", packet);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to get map: {}", e);
            return;
        }
    };

    map.junk_item(player_id, packet.junk_item.id, packet.junk_item.amount);
}
