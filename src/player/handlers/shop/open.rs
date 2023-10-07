use eo::{
    data::{EOChar, Serializeable, StreamReader},
    protocol::client::shop::Open,
};

use crate::player::PlayerHandle;

pub async fn open(reader: StreamReader, player: PlayerHandle) {
    let mut request = Open::default();
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
        map.open_shop(player_id, request.npc_index as EOChar);
    }
}
