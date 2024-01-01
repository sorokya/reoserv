use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{ChairRequestClientPacket, ChairRequestClientPacketSitActionData, SitAction},
        PacketAction,
    },
};

use crate::player::PlayerHandle;

async fn request(reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    let request = match ChairRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing ChairRequestClientPacket {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        match request.sit_action {
            SitAction::Sit => {
                let coords = match request.sit_action_data {
                    Some(ChairRequestClientPacketSitActionData::Sit(sit)) => sit.coords,
                    _ => {
                        error!("Sit action data is not sit");
                        return;
                    }
                };
                map.sit_chair(player_id, coords);
            }
            SitAction::Stand => map.stand(player_id),
            _ => {}
        }
    }
}

pub async fn chair(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet Chair_{:?}", action),
    }
}
