use eo::{
    data::{Serializeable, StreamBuilder},
    protocol::{PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

pub async fn request(player: PlayerHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        let nearby_info = map.get_nearby_info(player_id).await;
        let mut builder = StreamBuilder::new();
        nearby_info.serialize(&mut builder);
        player.send(
            PacketAction::Reply,
            PacketFamily::Refresh,
            builder.get(),
        );
    }

    Ok(())
}
