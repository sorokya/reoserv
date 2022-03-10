use eo::{
    data::{Serializeable, StreamReader},
    net::packets::server::login::Reply,
    net::{packets::client::login::Request, replies::LoginReply, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn request(
    buf: PacketBuf,
    player: PlayerHandle,
    mut world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!(
        "Recv: Request {{ name: {}, password: ******** }}",
        request.name
    );

    let reply = match world
        .login(
            player.clone(),
            request.name.clone(),
            request.password.clone(),
        )
        .await
    {
        Ok(reply) => reply,
        Err(e) => {
            error!("Login error: {}", e);
            Reply {
                reply: LoginReply::Busy,
                character_list: None,
            }
        }
    };

    debug!("Reply: {:?}", reply);

    player.send(Action::Reply, Family::Login, reply.serialize());

    Ok(())
}
