use eolib::protocol::net::{
    server::{PriestReply, PriestReplyServerPacket},
    PacketAction, PacketFamily,
};

use crate::{map::WeddingState, LANG, SETTINGS};

use super::super::Map;

const TICKS_PER_STATE: i32 = 3;

impl Map {
    pub fn timed_wedding(&mut self) {
        let (state, npc_index, player_id, partner_id) = match self.wedding.as_ref() {
            Some(wedding) => (
                wedding.state,
                wedding.npc_index,
                wedding.player_id,
                wedding.partner_id,
            ),
            None => return,
        };

        if matches!(
            state,
            WeddingState::Requested | WeddingState::AskPartner | WeddingState::AskPlayer
        ) {
            return;
        }

        if state == WeddingState::Accepted {
            self.npc_chat(
                npc_index,
                &get_lang_string!(
                    &LANG.wedding_start,
                    delay = SETTINGS.marriage.ceremony_start_delay_seconds
                ),
            );

            if let Some(wedding) = self.wedding.as_mut() {
                wedding.state = WeddingState::PriestDialog1;
            }
            return;
        }

        let wait_for = if state == WeddingState::PriestDialog1 {
            SETTINGS.marriage.ceremony_start_delay_seconds
        } else {
            TICKS_PER_STATE
        };

        if self.wedding_ticks >= wait_for {
            let character = match self.characters.get(&player_id) {
                Some(character) => character,
                None => {
                    self.npc_chat(npc_index, &LANG.wedding_error);
                    self.wedding = None;
                    self.wedding_ticks = 0;
                    return;
                }
            };

            let partner = match self.characters.get(&partner_id) {
                Some(character) => character,
                None => {
                    self.npc_chat(npc_index, &LANG.wedding_error);
                    self.wedding = None;
                    self.wedding_ticks = 0;
                    return;
                }
            };

            let (message, next_state) = match state {
                WeddingState::PriestDialog1 => (
                    get_lang_string!(
                        &LANG.wedding_one,
                        partner = partner.name,
                        name = character.name
                    ),
                    WeddingState::PriestDialog2,
                ),
                WeddingState::PriestDialog2 => (
                    get_lang_string!(
                        &LANG.wedding_two,
                        partner = partner.name,
                        name = character.name
                    ),
                    WeddingState::AskPartner,
                ),
                WeddingState::AskPartner => (
                    get_lang_string!(
                        &LANG.wedding_do_you,
                        partner = partner.name,
                        name = character.name
                    ),
                    WeddingState::AskPartner,
                ),
                _ => return,
            };

            self.npc_chat(npc_index, &message);
            if let Some(wedding) = self.wedding.as_mut() {
                self.wedding_ticks = 0;
                wedding.state = next_state;
            }

            if next_state == WeddingState::AskPartner {
                partner.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Priest,
                    &PriestReplyServerPacket {
                        reply_code: PriestReply::DoYou,
                    },
                );
            }

            if next_state == WeddingState::AskPlayer {
                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Priest,
                    &PriestReplyServerPacket {
                        reply_code: PriestReply::DoYou,
                    },
                );
            }
        }

        self.wedding_ticks += 1;
    }
}
