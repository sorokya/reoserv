use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::door,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn open(buf: PacketBuf, player: PlayerHandle) {
    let mut open = door::Open::default();
    let reader = StreamReader::new(&buf);
    open.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        let player_id = player.get_player_id().await;
        map.open_door(player_id, open.coords);
    }
}
