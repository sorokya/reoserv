use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::walk,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn player(buf: PacketBuf, player: PlayerHandle) {
    let mut packet = walk::Player::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await;
        map.walk(player_id, packet.timestamp, packet.coords, packet.direction);
    }
}
