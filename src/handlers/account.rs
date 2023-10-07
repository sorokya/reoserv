use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        client::account::{Create, Request},
        PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn create(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut create = Create::default();
    create.deserialize(&reader);

    match world.create_account(player.clone(), create.clone()).await {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
        }
        Err(e) => {
            player.close(format!("Create account failed: {}", e));
        }
    };
}

async fn request(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    if let Ok(reply) = world
        .request_account_creation(request.username, player.clone())
        .await
    {
        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
    }
}

pub async fn account(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Create => create(reader, player, world).await,
        PacketAction::Request => request(reader, player, world).await,
        _ => error!("Unhandled packet Account_{:?}", action),
    }
}
