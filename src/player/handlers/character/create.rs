use eo::{
    data::{Serializeable, StreamReader},
    net::{
        packets::{client::character::Create, server::character::Reply},
        replies::CharacterReply,
        Action, Family,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn create(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut create = Create::default();
    let reader = StreamReader::new(&buf);
    create.deserialize(&reader);

    debug!("Recv: {:?}", create);

    match world.create_character(create, player.clone()).await {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);
            player.send(Action::Reply, Family::Character, reply.serialize());
        }
        Err(e) => {
            error!("Create character failed: {}", e);
        }
    };
}
