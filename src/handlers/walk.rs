use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::client::WalkPlayerClientPacket,
};

use crate::player::PlayerHandle;

pub async fn walk(reader: EoReader, player: PlayerHandle) {
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

    let packet = match WalkPlayerClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing WalkPlayerClientPacket {}", e);
            return;
        }
    };

    map.walk(
        player_id,
        packet.walk_action.direction,
        packet.walk_action.coords,
        packet.walk_action.timestamp,
    );
}
