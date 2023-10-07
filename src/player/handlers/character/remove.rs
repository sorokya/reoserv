use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::character::Remove, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn remove(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut remove = Remove::default();
    remove.deserialize(&reader);

    debug!("Recv: {:?}", remove);

    match world
        .delete_character(remove.session_id, remove.character_id, player.clone())
        .await
    {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Reply, PacketFamily::Character, builder.get());
        }
        Err(e) => {
            error!("Delete character failed: {}", e);
        }
    };
}
