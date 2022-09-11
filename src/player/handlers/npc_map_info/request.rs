use eo::{
    data::{Serializeable, StreamReader},
    net::{packets::{client::npc_map_info::Request, server::npc_map_info::Reply}, Action, Family},
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let map_info = map
            .get_map_info(None, Some(request.npc_indexes))
            .await;
        if map_info.npcs.is_some() {
            let reply = Reply {
                npcs: map_info.npcs.unwrap(),
            };
            debug!("Reply: {:?}", reply);
            player.send(Action::Agree, Family::Npc, reply.serialize());
        }
    }
}
