use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::face::Player,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn player(buf: PacketBuf, player: PlayerHandle) {
    let mut packet = Player::default();
    let reader = StreamReader::new(&buf);
    packet.deserialize(&reader);

    debug!("Recv: {:?}", packet);

    let player_id = player.get_player_id().await;
    if let Ok(map) = player.get_map().await {
        map.face(player_id, packet.direction);
    }
}
