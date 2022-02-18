use std::sync::Arc;

use eo::{
    data::{EOShort, Serializeable, StreamReader},
    net::{
        packets::{
            client::welcome::Agree,
            server::init::{InitFileClass, InitFileItem, InitFileMap, InitFileNPC, InitFileSpell},
        },
        Action, Family,
    },
};

use tokio::sync::Mutex;

use crate::{player::Command, world::World, PacketBuf, Tx};

pub async fn agree(
    buf: PacketBuf,
    tx: &Tx,
    world: Arc<Mutex<World>>,
    map_id: EOShort,
) -> Result<(), Box<dyn std::error::Error>> {
    debug!("Welcome_Agree: {:?}", buf);
    let mut agree = Agree::default();
    let reader = StreamReader::new(&buf);
    agree.deserialize(&reader);

    debug!("Recv: {:?}", agree);

    info!("Player wants {:?} file", agree.file_type);

    let world = world.lock().await;

    match agree.file_type {
        eo::net::FileType::Map => {
            let mut reply = InitFileMap::new();
            let maps = world.maps.lock().unwrap();
            let map = maps.get(&map_id).unwrap();
            reply.data = map.serialize();
            tx.send(Command::Send(Action::Init, Family::Init, reply.serialize()))?;
        }
        eo::net::FileType::Item => {
            let mut reply = InitFileItem::new();
            let item_file = world.item_file.lock().unwrap();
            reply.id = 1;
            reply.data = item_file.serialize();
            tx.send(Command::Send(Action::Init, Family::Init, reply.serialize()))?;
        }
        eo::net::FileType::NPC => {
            let mut reply = InitFileNPC::new();
            let npc_file = world.npc_file.lock().unwrap();
            reply.id = 1;
            reply.data = npc_file.serialize();
            tx.send(Command::Send(Action::Init, Family::Init, reply.serialize()))?;
        }
        eo::net::FileType::Spell => {
            let mut reply = InitFileSpell::new();
            let spell_file = world.spell_file.lock().unwrap();
            reply.id = 1;
            reply.data = spell_file.serialize();
            tx.send(Command::Send(Action::Init, Family::Init, reply.serialize()))?;
        }
        eo::net::FileType::Class => {
            let mut reply = InitFileClass::new();
            let class_file = world.class_file.lock().unwrap();
            reply.id = 1;
            reply.data = class_file.serialize();
            tx.send(Command::Send(Action::Init, Family::Init, reply.serialize()))?;
        }
    }

    Ok(())
}
