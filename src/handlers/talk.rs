use eolib::{
    data::{EoReader, EoSerialize},
    protocol::{
        net::{
            client::{
                TalkAdminClientPacket, TalkAnnounceClientPacket, TalkMsgClientPacket,
                TalkOpenClientPacket, TalkReportClientPacket, TalkRequestClientPacket,
                TalkTellClientPacket,
            },
            PacketAction,
        },
        AdminLevel,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

use super::handle_command::handle_command;

async fn admin(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let admin = match TalkAdminClientPacket::deserialize(&reader) {
        Ok(admin) => admin,
        Err(e) => {
            error!("Error deserializing TalkAdminClientPacket {}", e);
            return;
        }
    };

    if let Ok(character) = player.get_character().await {
        if i32::from(character.admin_level) >= i32::from(AdminLevel::Guardian) {
            world.broadcast_admin_message(character.name, admin.message);
        }
    }
}

async fn announce(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let announce = match TalkAnnounceClientPacket::deserialize(&reader) {
        Ok(announce) => announce,
        Err(e) => {
            error!("Error deserializing TalkAnnounceClientPacket {}", e);
            return;
        }
    };

    if let Ok(character) = player.get_character().await {
        if i32::from(character.admin_level) >= i32::from(AdminLevel::Guardian) {
            world.broadcast_announcement(character.name, announce.message);
        }
    }
}

async fn msg(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let msg = match TalkMsgClientPacket::deserialize(&reader) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Error deserializing TalkMsgClientPacket {}", e);
            return;
        }
    };

    if let Ok(character) = player.get_character().await {
        world.broadcast_global_message(character.player_id.unwrap(), character.name, msg.message)
    }
}

async fn report(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let report = match TalkReportClientPacket::deserialize(&reader) {
        Ok(report) => report,
        Err(e) => {
            error!("Error deserializing TalkReportClientPacket {}", e);
            return;
        }
    };

    if let Ok(character) = player.get_character().await {
        if report.message.starts_with('$') && character.admin_level != AdminLevel::Player {
            let args: Vec<&str> = report.message[1..].split_whitespace().collect();
            if !args.is_empty() {
                handle_command(args.as_slice(), &character, player, world).await;
            }
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

async fn tell(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let tell = match TalkTellClientPacket::deserialize(&reader) {
        Ok(tell) => tell,
        Err(e) => {
            error!("Error deserializing TalkTellClientPacket {}", e);
            return;
        }
    };

    world.send_private_message(player, tell.name, tell.message);
}

fn open(reader: EoReader, player_id: i32, world: WorldHandle) {
    let open = match TalkOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing TalkOpenClientPacket {}", e);
            return;
        }
    };

    world.broadcast_party_message(player_id, open.message);
}

async fn request(reader: EoReader, player: PlayerHandle, world: WorldHandle) {
    let request = match TalkRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing TalkRequestClientPacket {}", e);
            return;
        }
    };

    let character = match player.get_character().await {
        Ok(character) => character,
        Err(e) => {
            error!("Error getting character: {}", e);
            return;
        }
    };

    let guild_tag = match character.guild_tag {
        Some(guild_tag) => guild_tag,
        None => return,
    };

    world.broadcast_guild_message(
        character.player_id,
        guild_tag,
        character.name,
        request.message,
    );
}

pub async fn talk(
    action: PacketAction,
    reader: EoReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    match action {
        PacketAction::Admin => admin(reader, player, world).await,
        PacketAction::Announce => announce(reader, player, world).await,
        PacketAction::Msg => msg(reader, player, world).await,
        PacketAction::Report => report(reader, player, world).await,
        PacketAction::Tell => tell(reader, player, world).await,
        PacketAction::Open => open(reader, player_id, world),
        PacketAction::Request => request(reader, player, world).await,
        _ => error!("Unhandled packet Talk_{:?}", action),
    }
}
