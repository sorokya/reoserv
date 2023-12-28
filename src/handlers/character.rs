use eo::{
    data::{i32, Serializeable, StreamReader},
    protocol::{
        client::character::{Create, Remove, Request, Take},
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn create(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut create = Create::default();
    create.deserialize(&reader);
    world.create_character(player_id, create);
}

fn remove(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut remove = Remove::default();
    remove.deserialize(&reader);
    world.delete_character(player_id, remove.session_id, remove.character_id);
}

fn request(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut request = Request::default();
    request.deserialize(&reader);

    if request.new != "NEW" {
        return;
    }

    world.request_character_creation(player_id);
}

fn take(reader: StreamReader, player_id: i32, world: WorldHandle) {
    let mut take = Take::default();
    take.deserialize(&reader);
    world.request_character_deletion(player_id, take.character_id);
}

pub async fn character(
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
        PacketAction::Create => create(reader, player_id, world),
        PacketAction::Remove => remove(reader, player_id, world),
        PacketAction::Request => request(reader, player_id, world),
        PacketAction::Take => take(reader, player_id, world),
        _ => error!("Unhandled packet Character_{:?}", action),
    }
}
