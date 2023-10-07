use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::client::shop::Buy,
};

use crate::player::PlayerHandle;

pub async fn buy(reader: StreamReader, player: PlayerHandle) {
    let mut request = Buy::default();
    request.deserialize(&reader);
    debug!("{:?}", request);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        map.buy_item(player_id, request.buy_item, request.session_id as EOShort);
    }
}
