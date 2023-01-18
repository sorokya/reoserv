use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::character::Take, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn take(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    let reader = StreamReader::new(&buf);
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    if let Ok(reply) = world
        .request_character_deletion(take.character_id, player.clone())
        .await
    {
        debug!("Reply: {:?}", reply);

        player.send(
            PacketAction::Player,
            PacketFamily::Character,
            reply.serialize(),
        );
    }
}
