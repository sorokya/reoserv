use eo::{
    data::{Serializeable, StreamReader, StreamBuilder, EOChar},
    protocol::{client::npcrange::Request, PacketAction, PacketFamily, server::npc},
};

use crate::{player::PlayerHandle, Bytes};

pub async fn request(buf: Bytes, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let map_info = map.get_map_info(Vec::default(), request.npc_indexes).await;
        if !map_info.nearby.npcs.is_empty() {
            let reply = npc::Agree {
                num_npcs: map_info.nearby.npcs.len() as EOChar,
                npcs: map_info.nearby.npcs,
            };
            debug!("Reply: {:?}", reply);
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Agree, PacketFamily::Npc, builder.get());
        }
    }
}
