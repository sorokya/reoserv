use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::range::Request, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        let reply = map
            .get_map_info(request.player_ids, request.npc_indexes)
            .await;

        if !reply.nearby.characters.is_empty() || !reply.nearby.npcs.is_empty() {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Range, builder.get());
        }
    }
}

pub async fn range(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet Range_{:?}", action),
    }
}
