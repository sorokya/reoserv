use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::{
        packets::{client::character::Request, server::character::Reply},
        replies::CharacterReply,
        Action, Family,
    },
};

use crate::{player::Command, PacketBuf, Tx};

pub async fn request(
    buf: PacketBuf,
    tx: &Tx,
    num_of_characters: EOChar,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    let mut reply = Reply::new();
    if request.message != "NEW" {
        reply.reply = CharacterReply::InvalidRequest;
        reply.message = "NO".to_string();
    }

    // TODO: configurable max number of characters?
    if num_of_characters >= 3 {
        reply.reply = CharacterReply::Full;
        reply.message = "NO".to_string();
    } else {
        reply.session_id = 1000; // TODO: sessions?
        reply.message = "OK".to_string();
    }

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Character,
        reply.serialize(),
    ))?;

    Ok(())
}
