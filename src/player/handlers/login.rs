use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::login::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn request(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    let reply = match world
        .login(
            player.clone(),
            request.username.clone(),
            request.password.clone(),
        )
        .await
    {
        Ok(reply) => reply,
        Err(e) => {
            player.close(format!("Login failed: {}", e));
            return;
        }
    };

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);

    player.send(PacketAction::Reply, PacketFamily::Login, builder.get());
}

pub async fn login(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Request => request(reader, player, world).await,
        _ => error!("Unhandled packet Login_{:?}", action),
    }
}
