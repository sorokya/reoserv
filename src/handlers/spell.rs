use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{SpellRequestClientPacket, SpellTargetOtherClientPacket, SpellTargetType},
        PacketAction,
    },
};

use crate::{character::SpellTarget, map::MapHandle, player::PlayerHandle};

fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match SpellRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing SpellRequestClientPacket {}", e);
            return;
        }
    };

    map.start_spell_chant(player_id, request.spell_id, request.timestamp);
}

fn target_self(_reader: EoReader, player_id: i32, map: MapHandle) {
    map.cast_spell(player_id, SpellTarget::Player);
}

fn target_other(reader: EoReader, player_id: i32, map: MapHandle) {
    let target_other = match SpellTargetOtherClientPacket::deserialize(&reader) {
        Ok(target_other) => target_other,
        Err(e) => {
            error!("Error deserializing SpellTargetOtherClientPacket {}", e);
            return;
        }
    };

    match target_other.target_type {
        SpellTargetType::Player => {
            map.cast_spell(player_id, SpellTarget::OtherPlayer(target_other.victim_id))
        }
        SpellTargetType::Npc => map.cast_spell(player_id, SpellTarget::Npc(target_other.victim_id)),
        _ => {}
    }
}

fn target_group(_reader: EoReader, player_id: i32, map: MapHandle) {
    map.cast_spell(player_id, SpellTarget::Group);
}

pub async fn spell(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
