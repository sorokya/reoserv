use eo::{
    data::{Serializeable, StreamReader, StreamBuilder},
    protocol::{client::welcome::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn request(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(reply) = world
        .select_character(request.character_id, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);
        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(
            PacketAction::Reply,
            PacketFamily::Welcome,
            builder.get(),
        );
    }
}
