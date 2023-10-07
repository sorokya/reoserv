use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::paperdoll,
};

use crate::player::PlayerHandle;

pub async fn remove(reader: StreamReader, player: PlayerHandle) {
    let mut packet = paperdoll::Remove::default();
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

    map.unequip(player_id, packet.item_id, packet.sub_loc);
}
