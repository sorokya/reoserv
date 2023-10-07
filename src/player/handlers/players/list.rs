use eo::{
    data::{Serializeable, StreamBuilder, StreamReader},
    protocol::{
        server::init::{Init, InitData, InitPlayers},
        InitReply, PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

pub async fn list(_: StreamReader, player: PlayerHandle, world: WorldHandle) {
    let players = world.get_online_list().await;

    let reply = Init {
        reply_code: InitReply::Players,
        data: InitData::Players(InitPlayers {
            num_online: players.len() as u16,
            list: players,
        }),
    };

    debug!("Reply: {:?}", reply);
    let mut builder = StreamBuilder::new();
    reply.serialize(&mut builder);
    player.send(PacketAction::Init, PacketFamily::Init, builder.get());
}
