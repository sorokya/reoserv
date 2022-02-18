use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::packets::server::login::Reply,
    net::{packets::client::login::Request, replies::LoginReply, Action, CharacterList, Family},
};
use mysql_async::Conn;
use sha2::{Digest, Sha256};

use crate::{
    player::{Command, State},
    utils::{get_account, get_character_list},
    PacketBuf, Tx,
};

pub async fn request(
    buf: PacketBuf,
    tx: &Tx,
    active_account_ids: Vec<u32>,
    conn: &mut Conn,
    salt: String,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!(
        "Recv: Request {{ name: {}, password: ******** }}",
        request.name
    );

    let hash_input = format!("{}{}{}", salt, request.name, request.password);
    let hash = Sha256::digest(hash_input.as_bytes());
    let mut reply = Reply::new();

    let (account_id, actual_hash) = match get_account(conn, &request.name).await {
        Ok(Some((account_id, actual_hash))) => (account_id, actual_hash),
        Ok(None) => {
            reply.reply = LoginReply::WrongUsername;
            debug!("Reply: {:?}", reply);
            tx.send(Command::Send(
                Action::Reply,
                Family::Login,
                reply.serialize(),
            ))?;
            return Ok(());
        }
        Err(e) => {
            error!("Failed to get account info: {}", e);
            reply.reply = LoginReply::Busy;
            debug!("Reply: {:?}", reply);
            tx.send(Command::Send(
                Action::Reply,
                Family::Login,
                reply.serialize(),
            ))?;
            return Ok(());
        }
    };

    if actual_hash != format!("{:x}", hash) {
        reply.reply = LoginReply::WrongPassword;
        debug!("Reply: {:?}", reply);
        tx.send(Command::Send(
            Action::Reply,
            Family::Login,
            reply.serialize(),
        ))?;
        return Ok(());
    }

    if active_account_ids.contains(&account_id) {
        reply.reply = LoginReply::LoggedIn;
        debug!("Reply: {:?}", reply);
        tx.send(Command::Send(
            Action::Reply,
            Family::Login,
            reply.serialize(),
        ))?;
        return Ok(());
    }

    // TODO: Ban check

    let characters = get_character_list(conn, account_id).await?;

    reply.reply = LoginReply::OK;
    reply.character_list = CharacterList {
        length: characters.len() as EOChar,
        unknown: 1,
        characters,
    };

    debug!("Reply: {:?}", reply);

    let num_of_characters = reply.character_list.length;
    tx.send(Command::SetState(State::LoggedIn(
        account_id,
        num_of_characters,
    )))?;
    tx.send(Command::Send(
        Action::Reply,
        Family::Login,
        reply.serialize(),
    ))?;

    Ok(())
}
