use eo::{
    data::{Serializeable, StreamReader, StreamBuilder},
    protocol::{client::playerrange::Request, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, Bytes};

pub async fn request(buf: Bytes, player: PlayerHandle) {
    let mut request = Request::default();
    let reader = StreamReader::new(buf);
    request.deserialize(&reader);

    debug!("Recv: {:?}", request);

    if let Ok(map) = player.get_map().await {
        let reply = map.get_map_info(request.player_ids, Vec::default()).await;
        if !reply.nearby.characters.is_empty() {
            debug!("Reply: {:?}", reply);
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Range, builder.get());
        }
    }
}
