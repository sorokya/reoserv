use eolib::protocol::{
    net::{PacketAction, PacketFamily, server::JukeboxMsgServerPacket},
    r#pub::SkillType,
};

use crate::{ITEM_DB, SETTINGS, SPELL_DB};

use super::super::Map;

impl Map {
    pub fn play_instrument(&mut self, player_id: i32, instrument_id: i32, note_id: i32) {
        if instrument_id <= 0 || note_id <= 0 || note_id > SETTINGS.load().bard.max_note_id {
            return;
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        if character.equipment.weapon == 0 {
            return;
        }

        if !SETTINGS
            .load()
            .bard
            .instrument_items
            .contains(&instrument_id)
        {
            return;
        }

        match ITEM_DB
            .load()
            .items
            .get(character.equipment.weapon as usize - 1)
        {
            Some(item_data) => {
                if item_data.spec1 != instrument_id {
                    return;
                }
            }
            None => return,
        };

        if !character.spells.iter().any(|s| {
            let spell_db = SPELL_DB.load();
            let spell_data = match spell_db.skills.get(s.id as usize - 1) {
                Some(spell) => spell,
                None => return false,
            };

            spell_data.r#type == SkillType::Bard
        }) {
            return;
        }

        self.send_packet_near_player(
            player_id,
            PacketAction::Msg,
            PacketFamily::Jukebox,
            &JukeboxMsgServerPacket {
                player_id,
                direction: character.direction,
                instrument_id,
                note_id,
            },
        );
    }
}
