use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::door::Open,
};

use crate::{player::PlayerHandle, Bytes};

pub async fn open(
    buf: Bytes,
    player: PlayerHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut open = Open::default();
    let reader = StreamReader::new(buf);
    open.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        map.open_door(player_id, open.coords);
    }

    Ok(())
}
