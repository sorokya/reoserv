use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::welcome::Agree, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn agree(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut agree = Agree::default();
    let reader = StreamReader::new(&buf);
    agree.deserialize(&reader);

    debug!("Recv: {:?}", agree);

    if let Ok(reply) = world.get_file(agree.file_type, player.clone()).await {
        debug!("Reply: {:?}", reply);
        player.send(Action::Init, Family::Init, reply.serialize());
    }
}
