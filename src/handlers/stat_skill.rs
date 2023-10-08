use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::statskill::{Add, AddData},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn add(reader: StreamReader, player: PlayerHandle) {
    let mut packet = Add::default();
    packet.deserialize(&reader);

    let player_id = player.get_player_id().await;
    if let Err(e) = player_id {
        error!("Error getting player id {}", e);
        return;
    }

    let player_id = player_id.unwrap();

    let map = player.get_map().await;
    if let Err(e) = map {
        error!("Error getting map {}", e);
        return;
    }

    let map = map.unwrap();

    match packet.data {
        AddData::Stat(stat) => map.level_stat(player_id, stat.stat_id),
        AddData::Skill(skill) => {
            error!("Unhandled packet StatSkill_Add_Skill {:?}", skill);
        }
        AddData::None => {}
    }
}

pub async fn stat_skill(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Add => add(reader, player).await,
        _ => error!("Unhandled packet StatSkill_{:?}", action),
    }
}
