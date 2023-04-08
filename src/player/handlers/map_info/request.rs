use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::range::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, Bytes};

pub async fn request(buf: Bytes, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let reply = map
            .get_map_info(request.player_ids, request.npc_indexes)
            .await;
        if !reply.nearby.characters.is_empty() || !reply.nearby.npcs.is_empty() {
            debug!("Reply: {:?}", reply);

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Range, builder.get());
        }
    }
}
