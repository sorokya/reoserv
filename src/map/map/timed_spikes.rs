use bytes::Bytes;
use eo::{
    data::EOShort,
    protocol::{PacketAction, PacketFamily},
    pubs::EmfTileSpec,
};

use super::Map;

impl Map {
    pub fn timed_spikes(&mut self) {
        if !self.has_timed_spikes || self.characters.is_empty() {
            return;
        }

        // TODO: only doing this to satisfy the borrow checker..
        let mut damaged_player_ids: Vec<EOShort> = Vec::new();

        for character in self.characters.values() {
            if self.get_tile(&character.coords).unwrap_or_default() == EmfTileSpec::TimedSpikes {
                damaged_player_ids.push(character.player_id.unwrap());
            } else {
                // TODO: only send if player near spike?
                character.player.as_ref().unwrap().send(
                    PacketAction::Report,
                    PacketFamily::Effect,
                    Bytes::from_static(b"S"),
                );
            }
        }

        for player_id in damaged_player_ids {
            self.spike_damage(player_id);
        }
    }
}
