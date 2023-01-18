use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::character::Remove, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn remove(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut remove = Remove::default();
    let reader = StreamReader::new(&buf);
    remove.deserialize(&reader);

    debug!("Recv: {:?}", remove);

    match world
        .delete_character(remove.session_id, remove.character_id, player.clone())
        .await
    {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);
            player.send(
                PacketAction::Reply,
                PacketFamily::Character,
                reply.serialize(),
            );
        }
        Err(e) => {
            error!("Delete character failed: {}", e);
        }
    };
}
