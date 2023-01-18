use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::walk::Admin,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn admin(buf: PacketBuf, player: PlayerHandle) {
    let mut packet = Admin::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await;
        map.walk(
            player_id,
            packet.walk.timestamp,
            packet.walk.coords,
            packet.walk.direction,
        );
    }
}
