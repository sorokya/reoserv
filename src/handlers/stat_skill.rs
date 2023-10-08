use eo::{
    data::{EOChar, EOShort, Serializeable, StreamReader},
    protocol::{
        client::statskill::{Add, AddData, Junk, Open, Remove, Take},
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

async fn add(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Add::default();
    packet.deserialize(&reader);

    match packet.data {
        AddData::Stat(stat) => map.level_stat(player_id, stat.stat_id),
        AddData::Skill(skill) => {
            error!("Unhandled packet StatSkill_Add_Skill {:?}", skill);
        }
        AddData::None => {}
    }
}

async fn junk(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Junk::default();
    packet.deserialize(&reader);
    map.reset_character(player_id, packet.session_id as EOShort);
}

async fn open(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Open::default();
    packet.deserialize(&reader);
    map.open_skill_master(player_id, packet.npc_index as EOChar);
}

async fn remove(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Remove::default();
    packet.deserialize(&reader);
    map.forget_skill(player_id, packet.spell_id, packet.session_id as EOShort);
}

async fn take(reader: StreamReader, player_id: EOShort, map: MapHandle) {
    let mut packet = Take::default();
    packet.deserialize(&reader);
    map.learn_skill(player_id, packet.spell_id, packet.session_id as EOShort);
}

pub async fn stat_skill(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Add => add(reader, player_id, map).await,
        PacketAction::Junk => junk(reader, player_id, map).await,
        PacketAction::Open => open(reader, player_id, map).await,
        PacketAction::Remove => remove(reader, player_id, map).await,
        PacketAction::Take => take(reader, player_id, map).await,
        _ => error!("Unhandled packet StatSkill_{:?}", action),
    }
}
