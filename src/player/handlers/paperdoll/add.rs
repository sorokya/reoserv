use bytes::Bytes;
use eo::{data::{StreamReader, Serializeable}, protocol::client::paperdoll};

use crate::player::PlayerHandle;

pub async fn add(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = paperdoll::Add::default();
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

    map.equip(player_id, packet.item_id, packet.sub_loc);
}