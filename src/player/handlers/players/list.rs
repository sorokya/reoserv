use eo::{
    data::Serializeable,
    protocol::{server::init::{InitData, InitPlayers, Init}, InitReply, PacketAction, PacketFamily},
};

use crate::{player::PlayerHandle, PacketBuf, world::WorldHandle};

pub async fn list(_buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let reply = Init {
        reply_code: InitReply::Players,
        data: InitData::Players(InitPlayers {
            num_online: players.len() as u16,
            list: players,
        })
    };

    debug!("Reply: {:?}", reply);
    player.send(PacketAction::Init, PacketFamily::Init, reply.serialize());
}
