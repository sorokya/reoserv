use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::walk::Spec,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn spec(buf: PacketBuf, player: PlayerHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Spec::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        map.walk(
            player_id,
            packet.walk.timestamp,
            packet.walk.coords,
            packet.walk.direction,
        );
    }

    Ok(())
}
