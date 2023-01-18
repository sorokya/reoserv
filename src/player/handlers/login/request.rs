use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::login::Request,
        server::login::{Reply, ReplyBusy, ReplyData},
        LoginReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn request(
    buf: PacketBuf,
    player: PlayerHandle,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!(
        "Recv: Request {{ name: {}, password: ******** }}",
        request.username
    );

    let reply = match world
        .login(
            player.clone(),
            request.username.clone(),
            request.password.clone(),
        )
        .await
    {
        Ok(reply) => reply,
        Err(e) => {
            error!("Login error: {}", e);
            Reply {
                reply_code: LoginReply::Busy,
                data: ReplyData::Busy(ReplyBusy {
                    no: "NO".to_string(),
                }),
            }
        }
    };

    debug!("Reply: {:?}", reply);

    player.send(PacketAction::Reply, PacketFamily::Login, reply.serialize());

    Ok(())
}
