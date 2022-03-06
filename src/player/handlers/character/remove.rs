use eo::{
    data::{Serializeable, StreamReader},
    net::{
        packets::{client::character::Remove, server::character::Reply},
        Action, Family, replies::CharacterReply,
    },
};

use crate::{
    player::PlayerHandle,
    PacketBuf, world::WorldHandle,
};

pub async fn remove(
    buf: PacketBuf,
    player: PlayerHandle,
    world: WorldHandle
) {
    let mut remove = Remove::default();
    let reader = StreamReader::new(&buf);
    remove.deserialize(&reader);

    debug!("Recv: {:?}", remove);

    let reply = match world.delete_character(remove.session_id, remove.character_id.into(), player.clone()).await {
        Ok(reply) => reply,
        Err(_) => Reply::no(CharacterReply::InvalidRequest),
    };

    debug!("Reply: {:?}", reply);

    player.send(
        Action::Reply,
        Family::Character,
        reply.serialize(),
    );
}
