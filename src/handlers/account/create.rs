use eo::{
    data::{Serializeable, StreamReader},
    net::packets::server::account::Reply,
    net::{packets::client::account::Create, replies::AccountReply, Action, Family},
};
use mysql_async::{prelude::*, Conn, Params, Row};
use sha2::{Digest, Sha256};

use crate::{player::Command, PacketBuf, Tx};

pub async fn create(
    buf: PacketBuf,
    tx: &Tx,
    conn: &mut Conn,
    salt: String,
    player_ip: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    let mut create = Create::default();
    let reader = StreamReader::new(&buf);
    create.deserialize(&reader);

    debug!("Recv: {:?}", create);

    let mut reply = Reply::new();
    if conn
        .exec_first::<Row, &str, Params>(
            include_str!("../login/get_account.sql"),
            params! {
                "name" => &create.name,
            },
        )
        .await?
        .is_some()
    {
        reply.reply = AccountReply::Exists;
        reply.message = "NO".to_string();
    } else {
        let hash_input = format!("{}{}{}", salt, create.name, create.password);
        let hash = Sha256::digest(hash_input.as_bytes());

        // TODO: validate name

        match conn
            .exec_drop(
                include_str!("create_account.sql"),
                params! {
                    "name" => &create.name,
                    "password_hash" => format!("{:x}", hash),
                    "real_name" => &create.fullname,
                    "location" => &create.location,
                    "email" => &create.email,
                    "computer" => &create.computer,
                    "hdid" => &create.hdid,
                    "register_ip" => &player_ip,
                },
            )
            .await
        {
            Ok(_) => {
                reply.reply = AccountReply::Created;
                reply.message = "YES".to_string();
                info!("New account: {}", create.name);
            }
            Err(e) => {
                // Not an ideal reply but I don't think the client has a "creation failed" handler
                reply.reply = AccountReply::NotApproved;
                reply.message = "NO".to_string();
                error!("Create account failed: {}", e);
            }
        }
    }

    debug!("Reply: {:?}", reply);

    tx.send(Command::Send(
        Action::Reply,
        Family::Account,
        reply.serialize(),
    ))?;

    Ok(())
}
