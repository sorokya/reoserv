use eo::{net::{packets::client::character_map_info::Request, Action, Family}, data::{Serializeable, StreamReader}};

use crate::{PacketBuf, player::PlayerHandle};

pub async fn request(
    buf: PacketBuf,
    player: PlayerHandle,
) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    match player.get_map().await {
        Ok(map) => {
            match map.get_character_map_info(request.player_id).await {
                Ok(map_info_reply) => {
                    debug!("Send: {:?}", map_info_reply);
                    player.send(Action::Reply, Family::MapInfo, map_info_reply.serialize());
                }
                Err(_) => {},
            }
        }
        Err(_) => {},
    }
}