use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::welcome::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(reply) = world
        .select_character(request.character_id, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);
        player.send(
            PacketAction::Reply,
            PacketFamily::Welcome,
            reply.serialize(),
        );
    }
}
