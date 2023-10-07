use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::client::shop::Create,
};

use crate::player::PlayerHandle;

pub async fn craft(reader: StreamReader, player: PlayerHandle) {
    let mut request = Create::default();
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
        map.craft_item(
            player_id,
            request.craft_item_id,
            request.session_id as EOShort,
        );
    }
}
