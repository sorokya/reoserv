use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        map::MapTileSpec,
        net::{server::EffectReportServerPacket, PacketAction, PacketFamily},
    },
};

use super::super::Map;

impl Map {
    pub fn timed_spikes(&mut self) {
        if !self.has_timed_spikes || self.characters.is_empty() {
            return;
        }

        // TODO: only doing this to satisfy the borrow checker..
        let mut damaged_player_ids: Vec<i32> = Vec::new();

        let packet = EffectReportServerPacket::new();

        let mut writer = EoWriter::new();
        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize EffectReportServerPacket: {}", e);
            return;
        }

        let buf = writer.to_byte_array();

        for character in self.characters.values() {
            if !character.hidden
                && self.get_tile(&character.coords).unwrap_or_default() == MapTileSpec::TimedSpikes
            {
                damaged_player_ids.push(character.player_id.unwrap());
            } else if let Some(player) = character.player.as_ref() {
                player.send_buf(PacketAction::Report, PacketFamily::Effect, buf.clone());
            }
        }

        for player_id in damaged_player_ids {
            self.spike_damage(player_id);
        }
    }
}
