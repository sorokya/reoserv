use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        client::warp::{Accept, Take},
        FileType, InitReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn accept(reader: StreamReader, player: PlayerHandle) {
    let mut accept = Accept::default();
    accept.deserialize(&reader);
    player.accept_warp(accept.map_id, accept.session_id);
}

async fn take(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    take.deserialize(&reader);

    if let Ok(mut reply) = world
        .get_file(FileType::Map, take.session_id, None, player.clone())
        .await
    {
        reply.reply_code = InitReply::WarpFileEmf;
        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(PacketAction::Init, PacketFamily::Init, builder.get());
    }
}

pub async fn warp(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Accept => accept(reader, player).await,
        PacketAction::Take => take(reader, player, world).await,
        _ => error!("Unhandled packet Warp_{:?}", action),
    }
}
