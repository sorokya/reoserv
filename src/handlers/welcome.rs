use eo::{
    data::{EOShort, Serializeable, StreamBuilder, StreamReader},
    protocol::{
        client::welcome::{Agree, AgreeData, Msg, Request},
        PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

async fn agree(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut agree = Agree::default();
    agree.deserialize(&reader);

    match world
        .get_file(
            agree.file_type,
            agree.session_id,
            match agree.data {
                AgreeData::Map(_) => None,
                AgreeData::Item(agree_item) => Some(agree_item.file_id),
                AgreeData::Npc(agree_npc) => Some(agree_npc.file_id),
                AgreeData::Spell(agree_spell) => Some(agree_spell.file_id),
                AgreeData::Class(agree_class) => Some(agree_class.file_id),
                AgreeData::None => unreachable!(),
            },
            player.clone(),
        )
        .await
    {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Init, PacketFamily::Init, builder.get());
        }
        Err(e) => {
            player.close(format!("Error getting file: {}", e));
        }
    }
}

async fn msg(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut msg = Msg::default();
    msg.deserialize(&reader);

    match world
        .enter_game(msg.session_id as EOShort, player.clone())
        .await
    {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Welcome, builder.get());
        }
        Err(e) => {
            player.close(format!("Error entering game: {}", e));
        }
    }
}

async fn request(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    match world
        .select_character(request.character_id, player.clone())
        .await
    {
        Ok(reply) => {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Welcome, builder.get());
        }
        Err(e) => {
            player.close(format!("Error selecting character: {}", e));
        }
    }
}

pub async fn welcome(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Agree => agree(reader, player, world).await,
        PacketAction::Msg => msg(reader, player, world).await,
        PacketAction::Request => request(reader, player, world).await,
        _ => error!("Unhandled packet Welcome_{:?}", action),
    }
}
