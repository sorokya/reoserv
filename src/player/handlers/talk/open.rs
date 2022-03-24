use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::talk::Open,
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn open(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut open = Open::default();
    let reader = StreamReader::new(&buf);
    open.deserialize(&reader);

    debug!("Recv: {:?}", open);
}
