use eo::{
    data::{EOChar, Serializeable, StreamReader},
    protocol::{
        client::talk::{Admin, Announce, Msg, Report, Tell},
        AdminLevel, PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

use super::handle_command::handle_command;

async fn admin(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut admin = Admin::default();
    admin.deserialize(&reader);

    if let Ok(character) = player.get_character().await {
        if character.admin_level as EOChar >= AdminLevel::Guardian as EOChar {
            world.broadcast_admin_message(character.name, admin.message);
        }
    }
}

async fn announce(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut announce = Announce::default();
    announce.deserialize(&reader);

    if let Ok(character) = player.get_character().await {
        if character.admin_level as EOChar >= AdminLevel::Guardian as EOChar {
            world.broadcast_announcement(character.name, announce.message);
        }
    }
}

async fn msg(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut message = Msg::default();
    message.deserialize(&reader);

    if let Ok(character) = player.get_character().await {
        world.broadcast_global_message(
            character.player_id.unwrap(),
            character.name,
            message.message,
        )
    }
}

async fn report(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut report = Report::default();
    report.deserialize(&reader);

    if let Ok(character) = player.get_character().await {
        if report.message.starts_with('$') && character.admin_level != AdminLevel::Player {
            let args: Vec<&str> = report.message[1..].split_whitespace().collect();
            handle_command(args.as_slice(), &character, player, world).await;
        } else if let Ok(map) = player.get_map().await {
            let player_id = match player.get_player_id().await {
                Ok(player_id) => player_id,
                Err(e) => {
                    error!("Failed to get player id: {}", e);
                    return;
                }
            };
            map.send_chat_message(player_id, report.message);
        }
    }
}

async fn tell(reader: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let mut tell = Tell::default();
    tell.deserialize(&reader);

    world.send_private_message(player, tell.name, tell.message);
}

pub async fn talk(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    match action {
        PacketAction::Admin => admin(reader, player, world).await,
        PacketAction::Announce => announce(reader, player, world).await,
        PacketAction::Msg => msg(reader, player, world).await,
        PacketAction::Report => report(reader, player, world).await,
        PacketAction::Tell => tell(reader, player, world).await,
        PacketAction::Open | PacketAction::Request => {} // no-op
        _ => error!("Unhandled packet Talk_{:?}", action),
    }
}
