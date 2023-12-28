use eo::{
    data::{i32, Serializeable, StreamBuilder, StreamReader},
    protocol::{client::npcrange::Request, server::npc, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        let map_info = map.get_map_info(Vec::default(), request.npc_indexes).await;
        if !map_info.nearby.npcs.is_empty() {
            let reply = npc::Agree {
                num_npcs: map_info.nearby.npcs.len() as i32,
                npcs: map_info.nearby.npcs,
            };

            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Agree, PacketFamily::Npc, builder.get());
        }
    }
}

pub async fn npc_range(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet NPCRange_{:?}", action),
    }
}
