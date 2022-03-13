use eo::{
    data::Serializeable,
    net::{Action, Family},
};

use crate::player::PlayerHandle;

pub async fn request(player: PlayerHandle) {
    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await;
        let nearby_info = map.get_nearby_info(player_id).await;
        player.send(Action::Reply, Family::Refresh, nearby_info.serialize());
    }
}
