use eolib::{data::{EoWriter, EoSerialize, EoReader}, protocol::net::{server::MessagePongServerPacket, PacketAction, PacketFamily}};

use crate::player::PlayerHandle;

fn ping(player: PlayerHandle) {
    let pong = MessagePongServerPacket::new();
    let mut writer = EoWriter::new();
    pong.serialize(&mut writer);
    player.send(PacketAction::Pong, PacketFamily::Message, writer.to_byte_array());
}

pub fn message(action: PacketAction, _reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Ping => ping(player),
        _ => error!("Unhandled packet Message_{:?}", action),
    }
}
