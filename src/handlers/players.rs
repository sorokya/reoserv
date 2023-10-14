use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        server::init::{Init, InitData, InitFriendListPlayers, InitPlayers},
        InitReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn accept(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let name = reader.get_break_string();

    let mut builder = StreamBuilder::new();
    builder.add_string(&name);

    let character = match world.get_character_by_name(name).await {
        Ok(character) => character,
        Err(_) => {
            player.send(PacketAction::Ping, PacketFamily::Players, builder.get());
            return;
        }
    };

    if character.hidden {
        player.send(PacketAction::Ping, PacketFamily::Players, builder.get());
        return;
    }

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(_) => {
            player.send(PacketAction::Ping, PacketFamily::Players, builder.get());
            return;
        }
    };

    let player_id = match character.player_id {
        Some(player_id) => player_id,
        None => {
            player.send(PacketAction::Ping, PacketFamily::Players, builder.get());
            return;
        }
    };

    let action = if map.has_player(player_id).await {
        PacketAction::Pong
    } else {
        PacketAction::Net3
    };

    player.send(action, PacketFamily::Players, builder.get());
}

pub async fn list(player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let reply = Init {
        reply_code: InitReply::FriendListPlayers,
        data: InitData::FriendListPlayers(InitFriendListPlayers {
            num_online: players.len() as u16,
            list: players.iter().map(|p| p.name.to_owned()).collect(),
        }),
    };

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());
}

pub async fn request(player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let reply = Init {
        reply_code: InitReply::Players,
        data: InitData::Players(InitPlayers {
            num_online: players.len() as u16,
            list: players,
        }),
    };

    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());
}

pub async fn players(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Accept => accept(reader, player, world).await,
        PacketAction::List => list(player, world).await,
        PacketAction::Request => request(player, world).await,
        _ => error!("Unhandled packet Players_{:?}", action),
    }
}
