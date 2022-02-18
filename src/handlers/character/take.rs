use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::{
        packets::{client::character::Take, server::character::Reply},
        replies::CharacterReply,
        Action, CharacterList, Family,
    },
};
use mysql_async::Conn;

use crate::{
    player::Command,
    utils::{delete_character, get_character_list},
    PacketBuf, Tx,
};

pub async fn take(
    buf: PacketBuf,
    tx: &Tx,
    conn: &mut Conn,
    account_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut take = Take::default();
    let reader = StreamReader::new(&buf);
    take.deserialize(&reader);

    debug!("Recv: {:?}", take);

    let mut reply = Reply::new();

    match delete_character(conn, take.character_id).await {
        Ok(_) => {
            reply.reply = CharacterReply::Deleted;
            reply.message = "YES".to_string();
            tx.send(Command::DeleteCharacter)?;
        }
        Err(e) => {
            // Not an ideal reply but I don't think the client has a "creation failed" handler
            reply.reply = CharacterReply::NotApproved;
            reply.message = "NO".to_string();
            error!("Delete character failed: {}", e);
        }
    }

    if reply.reply == CharacterReply::Deleted {
        let characters = get_character_list(conn, account_id).await?;
        reply.character_list = CharacterList {
            length: characters.len() as EOChar,
            unknown: 1,
            characters,
        };
    }

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Character,
        reply.serialize(),
    ))?;

    Ok(())
}
