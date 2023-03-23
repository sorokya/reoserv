use eo::{
    data::{Serializeable, StreamReader, StreamBuilder},
    protocol::{client::character::Take, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn take(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    let reader = StreamReader::new(buf);
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    if let Ok(reply) = world
        .request_character_deletion(take.character_id, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(
            PacketAction::Player,
            PacketFamily::Character,
            builder.get(),
        );
    }
}
