use eo::{
    data::{EOShort, Serializeable, StreamReader},
    protocol::{
        client::welcome::{Agree, AgreeData, Msg, Request},
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn agree(reader: StreamReader, player_id: EOShort, world: WorldHandle) {
    let mut agree = Agree::default();
    agree.deserialize(&reader);

    world.get_file(
        player_id,
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
        false,
    );
}

fn msg(reader: StreamReader, player_id: EOShort, world: WorldHandle) {
    let mut msg = Msg::default();
    msg.deserialize(&reader);
    world.enter_game(player_id, msg.session_id as EOShort);
}

fn request(reader: StreamReader, player_id: EOShort, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);
    world.select_character(player_id, request.character_id);
}

pub async fn welcome(
    action: PacketAction,
    reader: StreamReader,
    player: PlayerHandle,
    world: WorldHandle,
) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Error getting player id {}", e);
            return;
        }
    };

    match action {
        PacketAction::Agree => agree(reader, player_id, world),
        PacketAction::Msg => msg(reader, player_id, world),
        PacketAction::Request => request(reader, player_id, world),
        _ => error!("Unhandled packet Welcome_{:?}", action),
    }
}
