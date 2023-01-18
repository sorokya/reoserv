use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::npcrange::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let reply = map.get_map_info(Vec::default(), request.npc_indexes).await;
        if reply.nearby.npcs.len() > 0 {
            debug!("Reply: {:?}", reply);
            player.send(PacketAction::Agree, PacketFamily::Npc, reply.serialize());
        }
    }
}
