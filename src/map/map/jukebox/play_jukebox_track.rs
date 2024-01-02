use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::MapTileSpec,
        net::{
            server::{JukeboxAgreeServerPacket, JukeboxReplyServerPacket, JukeboxUseServerPacket},
            PacketAction, PacketFamily,
        },
    },
};

use crate::SETTINGS;

use super::super::Map;

impl Map {
    pub fn play_jukebox_track(&mut self, player_id: i32, track_id: i32) {
        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if !self.player_in_range_of_tile(player_id, MapTileSpec::Jukebox) {
            return;
        }

        debug!("Requesting track: {}", track_id);

        if self.jukebox_ticks > 0
            || character.get_item_amount(1) < SETTINGS.jukebox.cost
            || track_id < 1
            || track_id > SETTINGS.jukebox.max_track_id
        {
            let packet = JukeboxReplyServerPacket::new();

            let mut writer = EoWriter::new();

            if let Err(e) = packet.serialize(&mut writer) {
                error!("Failed to serialize JukeboxOpenServerPacket: {}", e);
                return;
            }

            character.player.as_ref().unwrap().send(
                PacketAction::Reply,
                PacketFamily::Jukebox,
                writer.to_byte_array(),
            );

            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, SETTINGS.jukebox.cost);
        self.jukebox_player = Some(character.name.clone());
        self.jukebox_ticks = SETTINGS.jukebox.track_timer;

        let packet = JukeboxAgreeServerPacket {
            gold_amount: character.get_item_amount(1),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize JukeboxAgreeServerPacket: {}", e);
            return;
        }

        character.player.as_ref().unwrap().send(
            PacketAction::Agree,
            PacketFamily::Jukebox,
            writer.to_byte_array(),
        );

        let packet = JukeboxUseServerPacket { track_id };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize JukeboxUseServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();
        for character in self.characters.values() {
            character.player.as_ref().unwrap().send(
                PacketAction::Use,
                PacketFamily::Jukebox,
                buf.clone(),
            );
        }
    }
}
