use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::packets::server::account::Reply,
    net::{packets::client::account::Request, replies::AccountReply, Action, Family},
};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{PacketBuf, world::WorldHandle, player::Command};

pub async fn request(
    buf: PacketBuf,
    player: UnboundedSender<Command>,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    let mut reply = Reply::new();

    let valid_name = world.validate_name(request.name.clone()).await;
    if !valid_name {
        reply.reply = AccountReply::NotApproved;
        reply.message = "NO".to_string();
        debug!("Reply: {:?}", reply);
        player.send(Command::Send(
            Action::Reply,
            Family::Account,
            reply.serialize(),
        ))?;
        return Ok(());
    }

    let name_in_use = world.account_name_in_use(request.name.clone()).await?;
    if name_in_use {
        reply.reply = AccountReply::Exists;
        reply.message = "NO".to_string();
        debug!("Reply: {:?}", reply);
        player.send(Command::Send(
            Action::Reply,
            Family::Account,
            reply.serialize(),
        ))?;
        return Ok(());
    }

    reply.session_id = 1000; // TODO: sessions?

    let (tx, rx) = oneshot::channel();
    let _ = player.send(Command::EnsureValidSequenceForAccountCreation { respond_to: tx });
    let _ = rx.await.unwrap();

    let (tx, rx) = oneshot::channel();
    let _ = player.send(Command::GetSequenceStart { respond_to: tx });
    let sequence_start = rx.await.unwrap();
    reply.sequence = sequence_start as EOChar;

    reply.message = "OK".to_string();

    debug!("Reply: {:?}", reply);

    player.send(Command::Send(
        Action::Reply,
        Family::Account,
        reply.serialize(),
    ))?;

    Ok(())
}
