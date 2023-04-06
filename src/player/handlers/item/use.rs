use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::item,
};

use crate::player::PlayerHandle;

pub async fn r#use(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = item::Use::default();
    packet.deserialize(&reader);

    debug!("{:?}", packet);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        },
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to get map: {}", e);
            return;
        },
    };

    map.use_item(player_id, packet.use_item_id);
}
