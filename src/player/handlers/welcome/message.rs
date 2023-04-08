use eo::{
    data::{EOShort, Serializeable, StreamBuilder, StreamReader},
    protocol::{client::welcome::Msg, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, world::WorldHandle, Bytes};

pub async fn message(buf: Bytes, player: PlayerHandle, world: WorldHandle) {
    let mut message = Msg::default();
    let reader = StreamReader::new(buf);
    message.deserialize(&reader);

    debug!("Recv: {:?}", message);

    match world
        .enter_game(message.session_id as EOShort, player.clone())
        .await
    {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);
            let mut builder = StreamBuilder::new();
            reply.serialize(&mut builder);
            player.send(PacketAction::Reply, PacketFamily::Welcome, builder.get());
        }
        Err(e) => {
            error!("{}", e);
        }
    }
}
