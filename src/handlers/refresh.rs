use eolib::protocol::net::PacketAction;

use crate::{map::MapHandle, player::PlayerHandle};

fn request(player_id: i32, map: MapHandle) {
    map.request_refresh(player_id);
}

pub async fn refresh(action: PacketAction, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
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

    match action {
        PacketAction::Request => request(player_id, map),
        _ => error!("Unhandled packet Refresh_{:?}", action),
    }
}
