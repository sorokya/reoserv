use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

async fn request(player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        let nearby_info = map.get_nearby_info(player_id).await;
        let mut builder = StreamBuilder::new();
        nearby_info.serialize(&mut builder);
        player.send(PacketAction::Reply, PacketFamily::Refresh, builder.get());
    }
}

pub async fn refresh(action: PacketAction, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(player).await,
        _ => error!("Unhandled packet Refresh_{:?}", action),
    }
}
