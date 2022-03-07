use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::character::Take, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn take(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut take = Take::default();
    let reader = StreamReader::new(&buf);
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    match world
        .request_character_deletion(take.character_id, player.clone())
        .await
    {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);

            player.send(Action::Player, Family::Character, reply.serialize());
        }
        Err(_) => {} // eat the error
    }
}
