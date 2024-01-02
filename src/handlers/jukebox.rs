use eolib::{
    data::{EoReader, EoSerialize},
    protocol::net::{client::JukeboxMsgClientPacket, PacketAction},
};

use crate::{map::MapHandle, player::PlayerHandle};

fn open(player_id: i32, map: MapHandle) {
    map.open_jukebox(player_id);
}

fn msg(reader: EoReader, player_id: i32, map: MapHandle) {
    let msg = match JukeboxMsgClientPacket::deserialize(&reader) {
        Ok(msg) => msg,
        Err(e) => {
            error!("Error deserializing JukeboxMsgClientPacket {}", e);
            return;
        }
    };

    map.play_jukebox_track(player_id, msg.track_id);
}

pub async fn jukebox(action: PacketAction, reader: EoReader, player: PlayerHandle) {
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
        PacketAction::Open => open(player_id, map),
        PacketAction::Msg => msg(reader, player_id, map),
        _ => error!("Unhandled packet Jukebox_{:?}", action),
    }
}
