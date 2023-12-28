use eo::{
    data::{i32, i32, Serializeable, StreamReader},
    protocol::{
        client::spell::{Request, TargetOther},
        PacketAction, SpellTargetType,
    },
};

use crate::{character::SpellTarget, map::MapHandle, player::PlayerHandle};

fn request(reader: StreamReader, player_id: i32, map: MapHandle) {
    let mut packet = Request::default();
    packet.deserialize(&reader);
    map.start_spell_chant(player_id, packet.spell_id, packet.timestamp);
}

fn target_self(_reader: StreamReader, player_id: i32, map: MapHandle) {
    map.cast_spell(player_id, SpellTarget::Player);
}

fn target_other(reader: StreamReader, player_id: i32, map: MapHandle) {
    let mut packet = TargetOther::default();
    packet.deserialize(&reader);

    match packet.target_type {
        SpellTargetType::Player => {
            map.cast_spell(player_id, SpellTarget::OtherPlayer(packet.victim_id))
        }
        SpellTargetType::Npc => {
            map.cast_spell(player_id, SpellTarget::Npc(packet.victim_id as i32))
        }
    }
}

fn target_group(_reader: StreamReader, player_id: i32, map: MapHandle) {
    map.cast_spell(player_id, SpellTarget::Group);
}

pub async fn spell(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
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
        PacketAction::Request => request(reader, player_id, map),
        PacketAction::TargetSelf => target_self(reader, player_id, map),
        PacketAction::TargetOther => target_other(reader, player_id, map),
        PacketAction::TargetGroup => target_group(reader, player_id, map),
        _ => error!("Unhandled packet Spell_{:?}", action),
    }
}
