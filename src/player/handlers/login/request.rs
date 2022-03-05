use eo::{
    data::{Serializeable, StreamReader},
    net::packets::server::login::Reply,
    net::{packets::client::login::Request, replies::LoginReply, Action, Family},
};
use sha2::{Digest};

use crate::{
    player::{PlayerHandle, State},
    world::WorldHandle,
    PacketBuf,
};

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

    let (reply, account_id) = match world
        .login(request.name.clone(), request.password.clone())
        .await
    {
        Ok((reply, account_id)) => (reply, account_id),
        Err(e) => {
            error!("Login error: {}", e);
            (
                Reply {
                    reply: LoginReply::Busy,
                    character_list: None,
                },
                0,
            )
        }
    };

    debug!("Reply: {:?}", reply);

    if reply.reply == LoginReply::OK {
        player.set_state(State::LoggedIn {
            account_id,
            num_of_characters: reply
                .character_list
                .as_ref()
                .expect("Reply is OK but character list is not set")
                .length,
        });
    }

    player.send(Action::Reply, Family::Login, reply.serialize());

    Ok(())
}
