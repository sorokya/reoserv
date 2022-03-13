use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::warp::Accept,
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn accept(buf: PacketBuf, player: PlayerHandle) {
    let mut accept = Accept::default();
    let reader = StreamReader::new(&buf);
    accept.deserialize(&reader);
    player.accept_warp(accept.map_id, accept.warp_id);
}
