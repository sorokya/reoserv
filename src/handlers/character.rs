use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        client::character::{Create, Remove, Request, Take},
        PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn create(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut create = Create::default();
    create.deserialize(&reader);

    match world.create_character(create, player.clone()).await {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Reply, PacketFamily::Character, builder.get());
        }
        Err(e) => {
            player.close(format!("Create character failed: {}", e));
        }
    };
}

async fn remove(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut remove = Remove::default();
    remove.deserialize(&reader);

    match world
        .delete_character(remove.session_id, remove.character_id, player.clone())
        .await
    {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Reply, PacketFamily::Character, builder.get());
        }
        Err(e) => {
            player.close(format!("Remove character failed: {}", e));
        }
    };
}

async fn request(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    if request.new != "NEW" {
        player.close("Invalid request".to_string());
        return;
    }

    let reply = match world.request_character_creation(player.clone()).await {
        Ok(reply) => reply,
        Err(e) => {
            player.close(format!("Request character failed: {}", e));
            return;
        }
    };

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);

    player.send(PacketAction::Reply, PacketFamily::Character, builder.get());
}

async fn take(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    take.deserialize(&reader);

    match world
        .request_character_deletion(take.character_id, player.clone())
        .await
    {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Player, PacketFamily::Character, builder.get());
        }
        Err(e) => {
            player.close(format!("Take character failed: {}", e));
        }
    }
}

pub async fn character(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Create => create(reader, player, world).await,
        PacketAction::Remove => remove(reader, player, world).await,
        PacketAction::Request => request(reader, player, world).await,
        PacketAction::Take => take(reader, player, world).await,
        _ => error!("Unhandled packet Character_{:?}", action),
    }
}
