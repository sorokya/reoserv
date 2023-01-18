use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::account::Create, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn create(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut create = Create::default();
    let reader = StreamReader::new(&buf);
    create.deserialize(&reader);

    debug!(
        "Recv: Create {{ session_id: {}, name: \"{}\", password: \"********\", fullname: \"{}\", location: \"{}\", email: \"{}\", computer: \"{}\", hdid: \"{}\" }}",
        create.session_id, create.username, create.fullname, create.location, create.email, create.computer, create.hdid
    );

    match world.create_account(player.clone(), create.clone()).await {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);
            player.send(
                PacketAction::Reply,
                PacketFamily::Account,
                reply.serialize(),
            );
        }
        Err(e) => {
            error!("Create account failed: {}", e);
        }
    };
}
