use eo::{
    data::{EOChar, Serializeable, StreamReader},
    protocol::{
        client::statskill::{Add, AddData, Open},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn add(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Add::default();
    packet.deserialize(&reader);

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

    match packet.data {
        AddData::Stat(stat) => map.level_stat(player_id, stat.stat_id),
        AddData::Skill(skill) => {
            error!("Unhandled packet StatSkill_Add_Skill {:?}", skill);
        }
        AddData::None => {}
    }
}

async fn open(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Open::default();
    packet.deserialize(&reader);

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

    map.open_skill_master(player_id, packet.npc_index as EOChar);
}

pub async fn stat_skill(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Add => add(reader, player).await,
        PacketAction::Open => open(reader, player).await,
        _ => error!("Unhandled packet StatSkill_{:?}", action),
    }
}
