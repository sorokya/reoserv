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

use crate::player::PlayerHandle;

fn create(reader: EoReader, player: PlayerHandle) {
    let create = match CharacterCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing CharacterCreateClientPacket {}", e);
            return;
        }
    };
    player.create_character(create);
}

fn remove(reader: EoReader, player: PlayerHandle) {
    let remove = match CharacterRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing CharacterRemoveClientPacket {}", e);
            return;
        }
    };
    player.delete_character(remove.session_id, remove.character_id);
}

fn request(reader: EoReader, player: PlayerHandle) {
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

    player.request_character_creation();
}

fn take(reader: EoReader, player: PlayerHandle) {
    let take = match CharacterTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing CharacterTakeClientPacket {}", e);
            return;
        }
    };
    player.request_character_deletion(take.character_id);
}

pub async fn character(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    match action {
        PacketAction::Create => create(reader, player),
        PacketAction::Remove => remove(reader, player),
        PacketAction::Request => request(reader, player),
        PacketAction::Take => take(reader, player),
        _ => error!("Unhandled packet Character_{:?}", action),
    }
}
