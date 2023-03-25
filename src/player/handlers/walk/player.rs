use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::walk::Player,
};

use crate::{player::PlayerHandle, Bytes};

pub async fn player(buf: Bytes, player: PlayerHandle) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Player::default();
    let reader = StreamReader::new(buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    // TODO: handle anti-speed

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await?;
        map.walk(
            player_id,
            packet.walk.direction,
        );
    }

    Ok(())
}
