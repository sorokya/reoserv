use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::welcome::Message, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn message(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut request = Message::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(reply) = world.enter_game(player.clone()).await {
        debug!("Reply: {:?}", reply);
        player.send(Action::Reply, Family::Welcome, reply.serialize());
    }
}
