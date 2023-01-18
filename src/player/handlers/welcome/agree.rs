use eo::{
    data::{Serializeable, StreamReader},
    protocol::{
        client::welcome::{Agree, AgreeData},
        PacketAction, PacketFamily,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle, PacketBuf};

pub async fn agree(buf: PacketBuf, player: PlayerHandle, world: WorldHandle) {
    let mut agree = Agree::default();
    let reader = StreamReader::new(&buf);
    agree.deserialize(&reader);

    debug!("Recv: {:?}", agree);

    match world
        .get_file(
            agree.file_type,
            agree.session_id,
            match agree.data {
                AgreeData::Map(_) => None,
                AgreeData::Item(agree_item) => Some(agree_item.file_id),
                AgreeData::Npc(agree_npc) => Some(agree_npc.file_id),
                AgreeData::Spell(agree_spell) => Some(agree_spell.file_id),
                AgreeData::Class(agree_class) => Some(agree_class.file_id),
                AgreeData::None => unreachable!(),
            },
            player.clone(),
        )
        .await
    {
        Ok(reply) => {
            debug!("Reply: {:?}", reply);
            player.send(PacketAction::Init, PacketFamily::Init, reply.serialize());
        }
        Err(e) => {
            error!("Error getting file: {}", e);
        }
    }
}
