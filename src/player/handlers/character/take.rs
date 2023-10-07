use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::character::Take, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn take(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    if let Ok(reply) = world
        .request_character_deletion(take.character_id, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);

        player.send(PacketAction::Player, PacketFamily::Character, builder.get());
    }
}
