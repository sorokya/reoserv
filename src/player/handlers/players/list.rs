use eo::{
    data::Serializeable,
    net::{Action, Family, packets::server::init::{Reply, ReplyPlayers}, replies::InitReply},
};

use crate::{player::PlayerHandle, PacketBuf, world::WorldHandle};

pub async fn list(_buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let players_reply = ReplyPlayers {
        players,
    };

    let reply = Reply {
        reply_code: InitReply::Players,
        reply: Box::new(players_reply),
    };

    debug!("Reply: {:?}", reply);
    player.send(Action::Init, Family::Init, reply.serialize());
}
