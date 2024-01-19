use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            GuildAcceptClientPacket, GuildCreateClientPacket, GuildKickClientPacket,
            GuildOpenClientPacket, GuildPlayerClientPacket, GuildRequestClientPacket,
            GuildUseClientPacket,
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

async fn player(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let packet = match GuildPlayerClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildPlayerClientPacket: {}", e);
            return;
        }
    };

    let session_id = match player.get_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Error getting player session id: {}", e);
            return;
        }
    };

    if session_id != packet.session_id {
        return;
    }

    map.request_to_join_guild(player_id, packet.guild_tag, packet.recruiter_name);
}

pub async fn r#use(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildUseClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildUseClientPacket: {}", e);
            return;
        }
    };

    player.accept_guild_join_request(packet.player_id);
}

pub fn kick(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildKickClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildKickClientPacket: {}", e);
            return;
        }
    };

    player.kick_guild_member(packet.session_id, packet.member_name);
}

pub async fn guild(action: PacketAction, reader: EoReader, player_handle: PlayerHandle) {
    let player_id = match player_handle.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Error getting player id: {}", e);
            return;
        }
    };

    let map = match player_handle.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting player map: {}", e);
            return;
        }
    };

    match action {
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Request => request(reader, player_handle),
        PacketAction::Accept => accept(reader, player_id, map),
        PacketAction::Create => create(reader, player_handle),
        PacketAction::Player => player(reader, player_id, player_handle, map).await,
        PacketAction::Use => r#use(reader, player_handle).await,
        PacketAction::Kick => kick(reader, player_handle),
        _ => error!("Unhandled packet Guild_{:?}", action),
    }
}
