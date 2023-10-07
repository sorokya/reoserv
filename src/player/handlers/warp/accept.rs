use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::warp::Accept,
};

use crate::player::PlayerHandle;

pub async fn accept(reader: StreamReader, player: PlayerHandle) {
    let mut accept = Accept::default();
    accept.deserialize(&reader);
    player.accept_warp(accept.map_id, accept.session_id);
}
