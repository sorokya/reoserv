use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            StatSkillAddClientPacket, StatSkillAddClientPacketActionTypeData,
            StatSkillJunkClientPacket, StatSkillOpenClientPacket, StatSkillRemoveClientPacket,
            StatSkillTakeClientPacket,
        },
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn add(reader: EoReader, player_id: i32, map: MapHandle) {
    let add = match StatSkillAddClientPacket::deserialize(&reader) {
        Ok(add) => add,
        Err(e) => {
            error!("Error deserializing StatSkillAddClientPacket {}", e);
            return;
        }
    };

    match add.action_type_data {
        Some(StatSkillAddClientPacketActionTypeData::Stat(stat)) => {
            map.level_stat(player_id, stat.stat_id)
        }
        Some(StatSkillAddClientPacketActionTypeData::Skill(skill)) => {
            error!("Unhandled packet StatSkill_Add_Skill {:?}", skill);
        }
        _ => {}
    }
}

async fn junk(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let junk = match StatSkillJunkClientPacket::deserialize(&reader) {
        Ok(junk) => junk,
        Err(e) => {
            error!("Error deserializing StatSkillJunkClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != junk.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.reset_character(player_id, npc_index);
}

async fn open(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let open = match StatSkillOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing StatSkillOpenClientPacket {}", e);
            return;
        }
    };

    let session_id = match player.generate_session_id().await {
        Ok(session_id) => session_id,
        Err(e) => {
            error!("Failed to generate session id: {}", e);
            return;
        }
    };

    map.open_skill_master(player_id, open.npc_index, session_id);
}

async fn remove(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let remove = match StatSkillRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing StatSkillRemoveClientPacket {}", e);
            return;
        }
    };

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != remove.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.forget_skill(player_id, npc_index, remove.spell_id);
}

async fn take(reader: EoReader, player_id: i32, player: PlayerHandle, map: MapHandle) {
    let take = match StatSkillTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing StatSkillTakeClientPacket {}", e);
            return;
        }
    };

    // Prevent learning new skills while trading
    if player.is_trading().await {
        return;
    }

    match player.get_session_id().await {
        Ok(session_id) => {
            if session_id != take.session_id {
                return;
            }
        }
        Err(_) => return,
    }

    let npc_index = match player.get_interact_npc_index().await {
        Some(npc_index) => npc_index,
        None => return,
    };

    map.learn_skill(player_id, npc_index, take.spell_id);
}

pub async fn stat_skill(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Error getting map {}", e);
            return;
        }
    };

    match action {
        PacketAction::Add => add(reader, player_id, map),
        PacketAction::Junk => junk(reader, player_id, player, map).await,
        PacketAction::Open => open(reader, player_id, player, map).await,
        PacketAction::Remove => remove(reader, player_id, player, map).await,
        PacketAction::Take => take(reader, player_id, player, map).await,
        _ => error!("Unhandled packet StatSkill_{:?}", action),
    }
}
