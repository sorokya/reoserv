use eo::{
    data::Serializeable,
    net::{packets::server::talk, replies::TalkReply, Action, Family},
};

use crate::player::PlayerHandle;

pub fn send_player_not_found(player: PlayerHandle, to: &str) {
    let packet = talk::Reply {
        reply: TalkReply::NotFound,
        name: to.to_string(),
    };
    let buf = packet.serialize();
    player.send(Action::Reply, Family::Talk, buf);
}
