use eo::{
    data::{Serializeable, StreamReader, EOChar},
    protocol::{
        client::shop::Open,
    },
};

use crate::{player::PlayerHandle, Bytes};

pub async fn open(
    buf: Bytes,
    player: PlayerHandle,
) {
    let mut request = Open::default();
    let reader = StreamReader::new(buf);
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
