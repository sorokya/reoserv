use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        client::login::Request,
        server::login::{Reply, ReplyBusy, ReplyData},
        LoginReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn request(
    buf: Bytes,
    player: PlayerHandle,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
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

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);

    player.send(PacketAction::Reply, PacketFamily::Login, builder.get());

    Ok(())
}
