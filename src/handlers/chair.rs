use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::chair::{Request, RequestData},
        PacketAction, SitAction,
    },
};

use crate::player::PlayerHandle;

async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    let coords = match request.data {
        RequestData::Sit(data) => data.coords,
        RequestData::None => {
            error!("No data in request");
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        match request.sit_action {
            SitAction::Sit => map.sit_chair(player_id, coords),
            SitAction::Stand => map.stand(player_id),
        }
    }
}

pub async fn chair(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet Chair_{:?}", action),
    }
}
