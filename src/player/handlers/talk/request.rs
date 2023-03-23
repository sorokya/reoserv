use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::talk::Request,
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn _request(buf: Bytes, _player: PlayerHandle, _world: WorldHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);
}
