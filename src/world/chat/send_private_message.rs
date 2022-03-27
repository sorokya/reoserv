use eo::{
    data::Serializeable,
    net::{packets::server::talk, Action, Family},
};

use crate::player::PlayerHandle;

pub fn send_private_message(from: &str, to: &PlayerHandle, message: &str) {
    let packet = talk::Tell {
        message: message.to_string(),
        name: from.to_string(),
    };
    let buf = packet.serialize();
    to.send(Action::Tell, Family::Talk, buf);
}
