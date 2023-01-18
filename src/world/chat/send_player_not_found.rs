use eo::{
    data::Serializeable,
    protocol::{server::talk, PacketAction, PacketFamily, TalkReply},
};

use crate::player::PlayerHandle;

pub fn send_player_not_found(player: PlayerHandle, to: &str) {
    let packet = talk::Reply {
        reply_code: TalkReply::NotFound,
        name: to.to_string(),
    };
    let buf = packet.serialize();
    player.send(PacketAction::Reply, PacketFamily::Talk, buf);
}
