use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::character::Create, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn create(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut create = Create::default();
    create.deserialize(&reader);

    debug!("Recv: {:?}", create);

    match world.create_character(create, player.clone()).await {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Reply, PacketFamily::Character, builder.get());
        }
        Err(e) => {
            error!("Create character failed: {}", e);
        }
    };
}
