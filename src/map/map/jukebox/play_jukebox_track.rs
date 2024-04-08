use eolib::protocol::{
    map::MapTileSpec,
    net::{
        server::{JukeboxAgreeServerPacket, JukeboxReplyServerPacket, JukeboxUseServerPacket},
        PacketAction, PacketFamily,
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

        if self.jukebox_ticks > 0
            || character.get_item_amount(1) < SETTINGS.jukebox.cost
            || track_id < 1
            || track_id > SETTINGS.jukebox.max_track_id
        {
            if let Some(player) = character.player.as_ref() {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Jukebox,
                    &JukeboxReplyServerPacket::new(),
                );
            }

            return;
        }

        let character = match self.characters.get_mut(&player_id) {
            Some(character) => character,
            None => return,
        };

        character.remove_item(1, SETTINGS.jukebox.cost);
        self.jukebox_player = Some(character.name.clone());
        self.jukebox_ticks = SETTINGS.jukebox.track_timer;

        if let Some(player) = character.player.as_ref() {
            player.send(
                PacketAction::Agree,
                PacketFamily::Jukebox,
                &JukeboxAgreeServerPacket {
                    gold_amount: character.get_item_amount(1),
                },
            );
        }

        self.send_packet_all(
            PacketAction::Use,
            PacketFamily::Jukebox,
            JukeboxUseServerPacket { track_id },
        );
    }
}
