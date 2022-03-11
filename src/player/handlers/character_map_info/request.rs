use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::client::character_map_info::Request, Action, Family},
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        if let Ok(map_info_reply) = map.get_map_info(Some(request.player_ids), None).await {
            debug!("Reply: {:?}", map_info_reply);
            player.send(Action::Reply, Family::CharacterMapInfo, map_info_reply.serialize());
        }
    }
}
