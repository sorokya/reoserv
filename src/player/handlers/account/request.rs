use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::account::Request, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(reply) = world
        .request_account_creation(request.name, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);

        player.send(Action::Reply, Family::Account, reply.serialize());
    }
}
