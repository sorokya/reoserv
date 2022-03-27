use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client::talk::Tell,
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn tell(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut tell = Tell::default();
    let reader = StreamReader::new(&buf);
    tell.deserialize(&reader);

    debug!("Recv: Tell {{ name: {}, message: ******** }}", tell.name);

    world.send_private_message(player, tell.name, tell.message);
}
