use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::account::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn request(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(reply) = world
        .request_account_creation(request.username, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
    }
}
