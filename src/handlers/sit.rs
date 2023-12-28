use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{SitAction, SitRequestClientPacket},
        PacketAction,
    },
};

use crate::{map::MapHandle, player::PlayerHandle};

fn request(reader: EoReader, player_id: i32, map: MapHandle) {
    let request = match SitRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing SitRequestClientPacket {}", e);
            return;
        }
    };

    match request.sit_action {
        SitAction::Sit => map.sit(player_id),
        SitAction::Stand => map.stand(player_id),
        _ => {}
    }
}

pub async fn sit(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to get map {}", e);
            return;
        }
    };

    match action {
        PacketAction::Request => request(reader, player_id, map),
        _ => error!("Unhandled packet Sit_{:?}", action),
    }
}
