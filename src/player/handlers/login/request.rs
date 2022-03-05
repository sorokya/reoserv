use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::packets::server::login::Reply,
    net::{packets::client::login::Request, replies::LoginReply, Action, CharacterList, Family},
};
use sha2::{Digest, Sha256};
use tokio::sync::mpsc::UnboundedSender;

use crate::{
    player::{Command, State},
    PacketBuf, world::{WorldHandle, LoginResult}, SETTINGS,
};

pub async fn request(
    buf: PacketBuf,
    player: UnboundedSender<Command>,
    mut world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!(
        "Recv: Request {{ name: {}, password: ******** }}",
        request.name
    );

    let hash_input = format!(
        "{}{}{}",
        SETTINGS.server.password_salt, request.name, request.password
    );
    let hash = Sha256::digest(hash_input.as_bytes());

    let result = world.login(request.name.clone(), format!("{:x}", hash)).await;
    let reply = match result {
        LoginResult::Success { account_id, character_list } => {
            player.send(Command::SetState(State::LoggedIn {
                account_id,
                num_of_characters: character_list.length,
            }))?;
            Reply {
                reply: LoginReply::OK,
                character_list: Some(character_list),
            }
        },
        LoginResult::LoggedIn => Reply { reply: LoginReply::LoggedIn, character_list: None },
        LoginResult::WrongUsername => Reply { reply: LoginReply::WrongUsername, character_list: None },
        LoginResult::WrongPassword => Reply { reply: LoginReply::WrongPassword, character_list: None },
        LoginResult::Err(_) => Reply { reply: LoginReply::Busy, character_list: None },
    };

    debug!("Reply: {:?}", reply);

    player.send(Command::Send(
        Action::Reply,
        Family::Login,
        reply.serialize(),
    ))?;

    Ok(())
}
