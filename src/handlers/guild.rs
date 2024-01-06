use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            GuildAcceptClientPacket, GuildCreateClientPacket, GuildOpenClientPacket,
            GuildRequestClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match GuildOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing GuildOpenClientPacket: {}", e);
            return;
        }
    };

    map.open_guild_master(player_id, open.npc_index);
}

fn request(reader: EoReader, player: PlayerHandle) {
    let request = match GuildRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing GuildRequestClientPacket: {}", e);
            return;
        }
    };

    player.request_guild_creation(request.session_id, request.guild_name, request.guild_tag);
}

fn accept(reader: EoReader, player_id: i32, map: MapHandle) {
    let accept = match GuildAcceptClientPacket::deserialize(&reader) {
        Ok(accept) => accept,
        Err(e) => {
            error!("Error deserializing GuildAcceptClientPacket: {}", e);
            return;
        }
    };

    map.accept_guild_creation_request(player_id, accept.inviter_player_id);
}

fn create(reader: EoReader, player: PlayerHandle) {
    let create = match GuildCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing GuildCreateClientPacket: {}", e);
            return;
        }
    };

    player.create_guild(
        create.session_id,
        create.guild_name,
        create.guild_tag,
        create.description,
    );
}

pub async fn guild(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id: {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting player map: {}", e);
            return;
        }
    };

    match action {
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Request => request(reader, player),
        PacketAction::Accept => accept(reader, player_id, map),
        PacketAction::Create => create(reader, player),
        _ => error!("Unhandled packet Guild_{:?}", action),
    }
}
