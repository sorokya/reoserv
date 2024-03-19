use eolib::{
    data::EoReader,
    protocol::net::{server::MessagePongServerPacket, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

fn ping(player: PlayerHandle) {
    player.send(
        PacketAction::Pong,
        PacketFamily::Message,
        &MessagePongServerPacket::new(),
    );
}

pub fn message(action: PacketAction, _reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Ping => ping(player),
        _ => error!("Unhandled packet Message_{:?}", action),
    }
}
