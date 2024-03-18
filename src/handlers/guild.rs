use eolib::{
    data::{EoReader, EoSerialize},
    protocol::{
        net::{
            client::{
                GuildAcceptClientPacket, GuildAgreeClientPacket, GuildBuyClientPacket,
                GuildCreateClientPacket, GuildJunkClientPacket, GuildKickClientPacket,
                GuildOpenClientPacket, GuildPlayerClientPacket, GuildRankClientPacket,
                GuildRemoveClientPacket, GuildReportClientPacket, GuildRequestClientPacket,
                GuildTakeClientPacket, GuildTellClientPacket, GuildUseClientPacket,
            },
            PacketAction,
        },
        r#pub::NpcType,
    },
};

use crate::{map::MapHandle, player::PlayerHandle, NPC_DB, SETTINGS};

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

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != packet.session_id {
                return;
            }
        }
        Err(e) => {
            error!("Error getting player session id: {}", e);
            return;
        }
    };

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

pub fn take(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildTakeClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildTakeClientPacket: {}", e);
            return;
        }
    };

    player.request_guild_info(packet.session_id, packet.info_type);
}

pub async fn buy(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let packet = match GuildBuyClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildBuyClientPacket: {}", e);
            return;
        }
    };

    if packet.gold_amount < SETTINGS.guild.min_deposit {
        return;
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != packet.session_id {
                return;
            }
        }
        Err(e) => {
            error!("Error getting player session id: {}", e);
            return;
        }
    };

    match map.get_npc_id_for_index(npc_index).await {
        Some(npc_id) => {
            let npc_data = match NPC_DB.npcs.get(npc_id as usize - 1) {
                Some(npc_data) => npc_data,
                None => return,
            };

            if npc_data.r#type != NpcType::Guild {
                return;
            }
        }
        None => return,
    }

    map.deposit_guild_gold(player_id, packet.gold_amount);
}

pub fn agree(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildAgreeClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildAgreeClientPacket: {}", e);
            return;
        }
    };

    let info_type_data = match packet.info_type_data {
        Some(info_type_data) => info_type_data,
        None => return,
    };

    player.update_guild(packet.session_id, info_type_data);
}

pub fn rank(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildRankClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildRankClientPacket: {}", e);
            return;
        }
    };

    player.assign_guild_rank(packet.session_id, packet.member_name, packet.rank);
}

pub fn report(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildReportClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildReportClientPacket: {}", e);
            return;
        }
    };

    player.request_guild_details(packet.session_id, packet.guild_identity);
}

pub fn tell(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildTellClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildTellClientPacket: {}", e);
            return;
        }
    };

    player.request_guild_memberlist(packet.session_id, packet.guild_identity);
}

pub fn remove(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildRemoveClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildRemoveClientPacket: {}", e);
            return;
        }
    };

    player.leave_guild(packet.session_id);
}

pub fn junk(reader: EoReader, player: PlayerHandle) {
    let packet = match GuildJunkClientPacket::deserialize(&reader) {
        Ok(packet) => packet,
        Err(e) => {
            error!("Error deserializing GuildJunkClientPacket: {}", e);
            return;
        }
    };

    player.disband_guild(packet.session_id);
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
        PacketAction::Take => take(reader, player_handle),
        PacketAction::Buy => buy(reader, player_id, player_handle, map).await,
        PacketAction::Agree => agree(reader, player_handle),
        PacketAction::Rank => rank(reader, player_handle),
        PacketAction::Report => report(reader, player_handle),
        PacketAction::Tell => tell(reader, player_handle),
        PacketAction::Remove => remove(reader, player_handle),
        PacketAction::Junk => junk(reader, player_handle),
        _ => error!("Unhandled packet Guild_{:?}", action),
    }
}
