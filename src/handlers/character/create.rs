use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::{
        packets::{client::character::Create, server::character::Reply},
        replies::CharacterReply,
        Action, CharacterList, Family,
    },
};
use mysql_async::Conn;

use crate::{
    handlers::utils::{create_character, get_character_list, CreateCharacterParams, character_exists},
    player::Command,
    PacketBuf, Tx,
};

pub async fn create(
    buf: PacketBuf,
    tx: &Tx,
    conn: &mut Conn,
    account_id: u32,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut create = Create::default();
    let reader = StreamReader::new(&buf);
    create.deserialize(&reader);

    debug!("Recv: {:?}", create);

    let mut reply = Reply::new();

    // TODO: validate name

    if character_exists(conn, &create.name).await? {
        reply.reply = CharacterReply::Exists;
        reply.message = "NO".to_string();
        tx.send(Command::Send(
            Action::Reply,
            Family::Character,
            reply.serialize(),
        ))?;
        return Ok(());
    }

    match create_character(
        conn,
        CreateCharacterParams {
            account_id,
            name: create.name.to_string(),
            gender: create.gender,
            race: create.race,
            hair_style: create.hair_style.into(),
            hair_color: create.hair_color.into(),
        },
    )
    .await
    {
        Ok(_) => {
            reply.reply = CharacterReply::Created;
            reply.message = "YES".to_string();
            info!("New character: {}", create.name);
            tx.send(Command::NewCharacter)?;
        }
        Err(e) => {
            // Not an ideal reply but I don't think the client has a "creation failed" handler
            reply.reply = CharacterReply::NotApproved;
            reply.message = "NO".to_string();
            error!("Create character failed: {}", e);
        }
    }

    if reply.reply == CharacterReply::Created {
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
