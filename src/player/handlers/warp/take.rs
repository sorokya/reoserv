use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::warp::Take, replies::InitReply, Action, Family, FileType},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn take(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    let reader = StreamReader::new(&buf);
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    if let Ok(mut reply) = world
        .get_file(FileType::Map, take.session_id, None, player.clone())
        .await
    {
        reply.reply_code = InitReply::WarpMap;
        debug!("Reply: {:?}", reply);
        player.send(Action::Init, Family::Init, reply.serialize());
    }
}
