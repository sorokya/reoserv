use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::face::Player,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn player(
    buf: PacketBuf,
    player: PlayerHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut packet = Player::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let player_id = player.get_player_id().await;
    let map = player.get_map().await?;
    map.face(player_id, packet.direction);

    Ok(())
}
