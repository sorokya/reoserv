use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{
        client::{
            CharacterCreateClientPacket, CharacterRemoveClientPacket, CharacterRequestClientPacket,
            CharacterTakeClientPacket,
        },
        PacketAction,
    },
};

use crate::{player::PlayerHandle, world::WorldHandle};

fn create(reader: EoReader, player_id: i32, world: WorldHandle) {
    let create = match CharacterCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing CharacterCreateClientPacket {}", e);
            return;
        }
    };
    world.create_character(player_id, create);
}

fn remove(reader: EoReader, player_id: i32, world: WorldHandle) {
    let remove = match CharacterRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing CharacterRemoveClientPacket {}", e);
            return;
        }
    };
    world.delete_character(player_id, remove.session_id, remove.character_id);
}

fn request(reader: EoReader, player_id: i32, world: WorldHandle) {
    let request = match CharacterRequestClientPacket::deserialize(&reader) {
        Ok(request) => request,
        Err(e) => {
            error!("Error deserializing CharacterRemoveClientPacket {}", e);
            return;
        }
    };

    if request.request_string != "NEW" {
        return;
    }

    world.request_character_creation(player_id);
}

fn take(reader: EoReader, player_id: i32, world: WorldHandle) {
    let take = match CharacterTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing CharacterTakeClientPacket {}", e);
            return;
        }
    };
    world.request_character_deletion(player_id, take.character_id);
}

pub async fn character(
    action: PacketAction,
    reader: EoReader,
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
