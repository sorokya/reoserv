use eolib::{data::EoReader, protocol::net::PacketAction};

use crate::player::PlayerHandle;

pub fn global(action: PacketAction, _reader: EoReader, _player: PlayerHandle) {
    match action {
        PacketAction::Open | PacketAction::Close => {} // no-op
        _ => error!("Unhandled packet Global_{:?}", action),
    }
}
