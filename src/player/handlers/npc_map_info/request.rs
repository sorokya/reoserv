use eo::{
    data::{EOChar, Serializeable, StreamBuilder, StreamReader},
    protocol::{client::npcrange::Request, server::npc, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

pub async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut request = Request::default();
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
