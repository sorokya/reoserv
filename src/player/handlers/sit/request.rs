use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::sit::Request, SitAction},
};

use crate::player::PlayerHandle;

pub async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);
    debug!("{:?}", request);

    let player_id = match player.get_player_id().await {
        Ok(id) => id,
        Err(e) => {
            error!("Failed to get player id {}", e);
            return;
        }
    };

    if let Ok(map) = player.get_map().await {
        match request.sit_action {
            SitAction::Sit => map.sit(player_id),
            SitAction::Stand => map.stand(player_id),
        }
    }
}
