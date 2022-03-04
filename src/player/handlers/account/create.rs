use eo::{
    data::{Serializeable, StreamReader},
    net::packets::server::account::Reply,
    net::{packets::client::account::Create, replies::AccountReply, Action, Family},
};
use lazy_static::lazy_static;
use sha2::{Digest, Sha256};
use tokio::sync::{mpsc::UnboundedSender, oneshot};

use crate::{player::Command, settings::Settings, world::WorldHandle, PacketBuf};

pub async fn create(
    buf: PacketBuf,
    player: UnboundedSender<Command>,
    world: WorldHandle,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    lazy_static! {
        static ref SETTINGS: Settings = Settings::new().expect("Failed to load settings!");
    };

    let mut create = Create::default();
    let reader = StreamReader::new(&buf);
    create.deserialize(&reader);

    debug!(
        "Recv: Create {{ session_id: {}, name: \"{}\", password: \"********\", fullname: \"{}\", location: \"{}\", email: \"{}\", computer: \"{}\", hdid: \"{}\" }}",
        create.session_id, create.name, create.fullname, create.location, create.email, create.computer, create.hdid
    );

    let mut reply = Reply::new();

    let valid_name = world.validate_name(create.name.clone()).await;
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

    let name_in_use = world.account_name_in_use(create.name.clone()).await?;
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

    let hash_input = format!(
        "{}{}{}",
        SETTINGS.server.password_salt, create.name, create.password
    );
    let hash = Sha256::digest(hash_input.as_bytes());

    let (tx, rx) = oneshot::channel();
    let _ = player.send(Command::GetIpAddr { respond_to: tx });
    let player_ip = rx.await.unwrap();

    match world
        .create_account(
            create.name.to_string(),
            format!("{:x}", hash),
            create.fullname,
            create.location,
            create.email,
            create.computer,
            create.hdid,
            player_ip.to_string(),
        )
        .await
    {
        Ok(_) => {
            reply.reply = AccountReply::Created;
            reply.message = "OK".to_string();
            info!("New account: {}", create.name);
        }
        Err(e) => {
            // Not an ideal reply but I don't think the client has a "creation failed" handler
            reply.reply = AccountReply::NotApproved;
            reply.message = "NO".to_string();
            error!("Create account failed: {}", e);
        }
    }

    debug!("Reply: {:?}", reply);

    player.send(Command::Send(
        Action::Reply,
        Family::Account,
        reply.serialize(),
    ))?;

    Ok(())
}
