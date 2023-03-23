use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::warp::Accept,
};

use crate::{player::PlayerHandle, Bytes};

pub async fn accept(buf: Bytes, player: PlayerHandle) {
    let mut accept = Accept::default();
    let reader = StreamReader::new(buf);
    accept.deserialize(&reader);
    player.accept_warp(accept.map_id, accept.session_id);
}
