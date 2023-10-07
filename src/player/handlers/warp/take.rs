use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::warp::Take, FileType, InitReply, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn take(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    if let Ok(mut reply) = world
        .get_file(FileType::Map, take.session_id, None, player.clone())
        .await
    {
        reply.reply_code = InitReply::WarpFileEmf;
        debug!("Reply: {:?}", reply);
        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(PacketAction::Init, PacketFamily::Init, builder.get());
    }
}
