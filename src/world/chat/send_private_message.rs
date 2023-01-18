use eo::{
    data::Serializeable,
    protocol::{server::talk, PacketAction, PacketFamily},
};

use crate::player::PlayerHandle;

pub fn send_private_message(from: &str, to: &PlayerHandle, message: &str) {
    let packet = talk::Tell {
        message: message.to_string(),
        player_name: from.to_string(),
    };
    let buf = packet.serialize();
    to.send(PacketAction::Tell, PacketFamily::Talk, buf);
}
