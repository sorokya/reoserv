use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{client::playerrange::Request, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

async fn request(reader: StreamReader, player: PlayerHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    if let Ok(map) = player.get_map().await {
        // TODO: Consider just doing this from inside the map itself
        // e.g map.player_range_request(player_id, player_ids);
        let reply = map.get_map_info(request.player_ids, Vec::default()).await;
        if !reply.nearby.characters.is_empty() {
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Range, builder.get());
        }
    }
}

pub async fn player_range(action: PacketAction, reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Request => request(reader, player).await,
        _ => error!("Unhandled packet PlayerRange_{:?}", action),
    }
}
