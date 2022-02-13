use eo::{
    character::{AdminLevel, Gender, Race},
    data::{EOChar, Serializeable, StreamReader},
    net::packets::server::login::Reply,
    net::{
        packets::client::login::Request, replies::LoginReply, Action, CharacterInfo, CharacterList,
        Family, PaperdollBAHSW,
    },
};
use mysql_async::{prelude::*, Conn, Params, Row};
use num_traits::FromPrimitive;
use sha2::{Digest, Sha256};

use crate::{player::Command, PacketBuf, Tx};

pub async fn request(
    buf: PacketBuf,
    tx: &Tx,
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
    let account_row = match conn
        .exec_first::<Row, &str, Params>(
            include_str!("get_account.sql"),
            params! {
                "name" => &request.name,
            },
        )
        .await?
    {
        Some(row) => row,
        _ => {
            reply.reply = LoginReply::WrongUsername;
            debug!("Reply: {:?}", reply);
            tx.send(Command::Send(
                Action::Reply,
                Family::Login,
                reply.serialize(),
            ))?;
            return Ok(());
        }
    };

    let actual_hash: String = account_row.get(0).unwrap();
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

    // TODO: Ban check

    let characters = conn
        .exec_map(
            include_str!("get_character_list.sql"),
            params! {
                "name" => &request.name,
            },
            |row: Row| CharacterInfo {
                id: row.get(0).unwrap(),
                name: row.get(1).unwrap(),
                level: row.get(2).unwrap(),
                gender: Gender::from_u8(row.get(3).unwrap()).unwrap(),
                hair_style: row.get(4).unwrap(),
                hair_color: row.get(5).unwrap(),
                race: Race::from_u8(row.get(6).unwrap()).unwrap(),
                admin_level: AdminLevel::from_u8(row.get(7).unwrap()).unwrap(),
                paperdoll: PaperdollBAHSW {
                    boots: row.get(8).unwrap(),
                    armor: row.get(9).unwrap(),
                    hat: row.get(10).unwrap(),
                    shield: row.get(11).unwrap(),
                    weapon: row.get(12).unwrap(),
                },
            },
        )
        .await?;

    reply.reply = LoginReply::OK;
    reply.character_list = CharacterList {
        length: characters.len() as EOChar,
        unknown: 1,
        characters,
    };

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Login,
        reply.serialize(),
    ))?;

    Ok(())
}
