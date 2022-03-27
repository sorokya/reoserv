use eo::{
    data::{EOShort, Serializeable, StreamReader},
    net::{packets::client::welcome::Message, Action, Family},
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn message(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut message = Message::default();
    let reader = StreamReader::new(&buf);
    message.deserialize(&reader);

    debug!("Recv: {:?}", message);

    match world
        .enter_game(message.session_id as EOShort, player.clone())
        .await
    {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);
            player.send(Action::Reply, Family::Welcome, reply.serialize());
        }
        Err(e) => {
            error!("{}", e);
        }
    }
}
