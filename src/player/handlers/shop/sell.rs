use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::client::shop::Sell,
};

use crate::player::PlayerHandle;

pub async fn sell(reader: StreamReader, player: PlayerHandle) {
    let mut request = Sell::default();
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
        map.sell_item(player_id, request.sell_item, request.session_id as EOShort);
    }
}
