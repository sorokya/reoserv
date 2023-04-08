use crate::player::PlayerHandle;
use bytes::Bytes;
use eo::{
    data::{Serializeable, StreamReader},
    protocol::client::attack::Use,
};

pub async fn r#use(buf: Bytes, player: PlayerHandle) {
    let reader = StreamReader::new(buf);
    let mut packet = Use::default();
    packet.deserialize(&reader);

    debug!("{:?}", packet);

    // TODO: implement anti speed

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

    map.attack(player_id, packet.direction);
}
