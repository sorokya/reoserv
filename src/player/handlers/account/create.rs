use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::account::Create, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn create(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut create = Create::default();
    let reader = StreamReader::new(buf);
    create.deserialize(&reader);

    debug!(
        "Recv: Create {{ session_id: {}, name: \"{}\", password: \"********\", fullname: \"{}\", location: \"{}\", email: \"{}\", computer: \"{}\", hdid: \"{}\" }}",
        create.session_id, create.username, create.fullname, create.location, create.email, create.computer, create.hdid
    );

    match world.create_account(player.clone(), create.clone()).await {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);

            player.send(PacketAction::Reply, PacketFamily::Account, builder.get());
        }
        Err(e) => {
            error!("Create account failed: {}", e);
        }
    };
}
