use eo::{
    data::{Serializeable, StreamReader},
    protocol::{client::range::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, PacketBuf};

pub async fn request(buf: PacketBuf, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(&buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let reply = map
            .get_map_info(request.player_ids, request.npc_indexes)
            .await;
        if reply.nearby.characters.len() > 0 || reply.nearby.npcs.len() > 0 {
            debug!("Reply: {:?}", reply);
            player.send(PacketAction::Reply, PacketFamily::Range, reply.serialize());
        }
    }
}
