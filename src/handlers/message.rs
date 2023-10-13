use eo::{
    data::{StreamBuilder, StreamReader},
    protocol::{PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

fn ping(player: PlayerHandle) {
    let mut builder = StreamBuilder::new();
    builder.add_short(2);
    player.send(PacketAction::Pong, PacketFamily::Message, builder.get());
}

pub async fn message(action: PacketAction, _reader: StreamReader, player: PlayerHandle) {
    match action {
        PacketAction::Ping => ping(player),
        _ => error!("Unhandled packet Message_{:?}", action),
    }
}
