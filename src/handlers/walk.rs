use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::walk,
};

use crate::player::PlayerHandle;

pub async fn walk(reader: StreamReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting map {}", e);
            return;
        }
    };

    let mut packet = walk::Player::default();
    packet.deserialize(&reader);

    map.walk(
        player_id,
        packet.walk.direction,
        packet.walk.coords,
        packet.walk.timestamp,
    );
}
