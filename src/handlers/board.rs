use eolib::{data::{EoReader, EoSerialize}, protocol::net::{client::{BoardCreateClientPacket, BoardOpenClientPacket, BoardRemoveClientPacket, BoardTakeClientPacket}, PacketAction}};

use crate::{map::MapHandle, player::PlayerHandle};

fn create(reader: EoReader, player_id: i32, map: MapHandle) {
    let create = match BoardCreateClientPacket::deserialize(&reader) {
        Ok(create) => create,
        Err(e) => {
            error!("Error deserializing BoardCreateClientPacket {}", e);
            return;
        }
    };

    map.create_board_post(player_id, create.post_subject, create.post_body);
}

fn open(reader: EoReader, player_id: i32, map: MapHandle) {
    let open = match BoardOpenClientPacket::deserialize(&reader) {
        Ok(open) => open,
        Err(e) => {
            error!("Error deserializing BoardOpenClientPacket {}", e);
            return;
        }
    };
    map.open_board(player_id, open.board_id + 1);
}

fn remove(reader: EoReader, player_id: i32, map: MapHandle) {
    let remove = match BoardRemoveClientPacket::deserialize(&reader) {
        Ok(remove) => remove,
        Err(e) => {
            error!("Error deserializing BoardOpenClientPacket {}", e);
            return;
        }
    };
    map.remove_board_post(player_id, remove.post_id);
}

fn take(reader: EoReader, player_id: i32, map: MapHandle) {
    let take = match BoardTakeClientPacket::deserialize(&reader) {
        Ok(take) => take,
        Err(e) => {
            error!("Error deserializing BoardTakeClientPacket {}", e);
            return;
        }
    };
    map.view_board_post(player_id, take.post_id);
}

pub async fn board(action: PacketAction, reader: EoReader, player: PlayerHandle) {
    let player_id = match player.get_player_id().await {
        Ok(player_id) => player_id,
        Err(e) => {
            error!("Failed to get player id: {}", e);
            return;
        }
    };

    let map = match player.get_map().await {
        Ok(map) => map,
        Err(e) => {
            error!("Failed to get map: {}", e);
            return;
        }
    };

    match action {
        PacketAction::Create => create(reader, player_id, map),
        PacketAction::Open => open(reader, player_id, map),
        PacketAction::Remove => remove(reader, player_id, map),
        PacketAction::Take => take(reader, player_id, map),
        _ => error!("Unhandled packet Board_{:?}", action),
    }
}
