use eo::{
    data::{Serializeable, StreamReader},
    net::packets::server::account::Reply,
    net::{packets::client::account::Create, replies::AccountReply, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn create(
    buf: PacketBuf,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let mut create = Create::default();
    let reader = StreamReader::new(&buf);
    create.deserialize(&reader);

    debug!(
        "Recv: Create {{ session_id: {}, name: \"{}\", password: \"********\", fullname: \"{}\", location: \"{}\", email: \"{}\", computer: \"{}\", hdid: \"{}\" }}",
        create.session_id, create.name, create.fullname, create.location, create.email, create.computer, create.hdid
    );

    let player_ip = player.get_ip_addr().await;
    let reply = match world.create_account(create.clone(), player_ip).await {
        Ok(reply) => reply,
        Err(e) => {
            error!("Create account failed: {}", e);
            // Not an ideal reply but I don't think the client has a "creation failed" handler
            Reply::no(AccountReply::NotApproved)
        }
    };

    debug!("Reply: {:?}", reply);

    player.send(Action::Reply, Family::Account, reply.serialize());
}
