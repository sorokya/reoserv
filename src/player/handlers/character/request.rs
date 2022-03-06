use eo::{
    data::{Serializeable, StreamReader},
    net::{
        packets::{client::character::Request, server::character::Reply},
        replies::CharacterReply,
        Action, Family,
    },
};

use crate::{player::PlayerHandle, PacketBuf, world::WorldHandle};

pub async fn request(
    buf: PacketBuf,
    player: PlayerHandle,
    world: WorldHandle
) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    let reply = if request.message != "NEW" {
        Reply::no(CharacterReply::InvalidRequest)
    } else {
        match world.request_character_creation(player.clone()).await {
            Ok(reply) => reply,
            Err(_) => Reply::no(CharacterReply::InvalidRequest),
        }
    };

    debug!("Reply: {:?}", reply);

    player.send(
        Action::Reply,
        Family::Character,
        reply.serialize(),
    );
}
