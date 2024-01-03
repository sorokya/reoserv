use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{GuildOpenClientPacket, GuildRequestClientPacket},
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

async fn request(reader: EoReader, player: PlayerHandle, player_id: i32, map: MapHandle) {
    let request = match GuildRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing GuildRequestClientPacket: {}", e);
            return;
        }
    };

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    let session_id = match player.get_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Error getting player session id: {}", e);
            return;
        }
    };

    if request.session_id != session_id {
        return;
    }

    map.request_guild_creation(player_id, npc_index, request.guild_tag, request.guild_name);
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
        PacketAction::Request => request(reader, player, player_id, map).await,
        _ => error!("Unhandled packet Guild_{:?}", action),
    }
}
