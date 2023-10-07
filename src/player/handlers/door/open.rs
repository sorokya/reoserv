use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::door::Open,
};

use crate::player::PlayerHandle;

pub async fn open(
    reader: StreamReader,
    player: PlayerHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut open = Open::default();
    open.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        map.open_door(player_id, open.coords);
    }

    Ok(())
}
