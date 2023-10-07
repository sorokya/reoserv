use eo::{data::StreamReader, protocol::PacketAction};

use crate::player::PlayerHandle;

pub async fn global(action: PacketAction, _reader: StreamReader, _player: PlayerHandle) {
    match action {
        PacketAction::Open | PacketAction::Close => {} // no-op
        _ => error!("Unhandled packet Global_{:?}", action),
    }
}
