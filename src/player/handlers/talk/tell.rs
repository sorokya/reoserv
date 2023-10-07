use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::talk::Tell,
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn tell(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut tell = Tell::default();
    tell.deserialize(&reader);

    debug!("Recv: Tell {{ name: {}, message: ******** }}", tell.name);

    world.send_private_message(player, tell.name, tell.message);
}
