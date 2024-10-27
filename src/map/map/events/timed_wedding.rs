use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{JukeboxPlayerServerPacket, PriestReply, PriestReplyServerPacket},
        PacketAction, PacketFamily,
    },
};

use crate::{map::WeddingState, LANG, SETTINGS};

use super::super::Map;

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

        if state == WeddingState::Requested {
            return;
        }

        let wait_for = match state {
            WeddingState::Accepted | WeddingState::PlayerAgrees | WeddingState::PartnerAgrees => 0,
            WeddingState::PriestDialog5AndConfetti => 2,
            WeddingState::PriestDialog1 => SETTINGS.marriage.ceremony_start_delay_seconds,
            WeddingState::AskPlayer | WeddingState::AskPartner | WeddingState::PriestDialog3 => 3,
            WeddingState::WaitingForPlayer | WeddingState::WaitingForPartner => 20,
            _ => 9,
        };

        self.wedding_ticks += 1;

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

            let next_state = match state {
                WeddingState::Accepted => {
                    self.npc_chat(
                        npc_index,
                        &get_lang_string!(
                            &LANG.wedding_start,
                            delay = SETTINGS.marriage.ceremony_start_delay_seconds
                        ),
                    );

                    let packet = JukeboxPlayerServerPacket {
                        mfx_id: SETTINGS.marriage.mfx_id,
                    };

                    let mut writer = EoWriter::new();

                    if let Err(e) = packet.serialize(&mut writer) {
                        error!("Failed to serialize JukeboxPlayerServerPacket: {}", e);
                        return;
                    }

                    let buf = writer.to_byte_array();

                    for character in self.characters.values() {
                        let player = match character.player {
                            Some(ref player) => player,
                            None => continue,
                        };

                        player.send_buf(PacketAction::Player, PacketFamily::Jukebox, buf.clone());
                    }

                    WeddingState::PriestDialog1
                }
                WeddingState::PriestDialog1 => {
                    self.npc_chat(
                        npc_index,
                        &get_lang_string!(
                            &LANG.wedding_one,
                            partner = partner.name,
                            name = character.name
                        ),
                    );
                    WeddingState::PriestDialog2
                }
                WeddingState::PriestDialog2 => {
                    self.npc_chat(
                        npc_index,
                        &get_lang_string!(
                            &LANG.wedding_two,
                            partner = partner.name,
                            name = character.name
                        ),
                    );
                    WeddingState::PriestDoYouPartner
                }
                WeddingState::PriestDoYouPartner => {
                    self.npc_chat(
                        npc_index,
                        &get_lang_string!(
                            &LANG.wedding_do_you,
                            partner = partner.name,
                            name = character.name
                        ),
                    );
                    WeddingState::AskPartner
                }
                WeddingState::AskPartner => {
                    let player = match partner.player.as_ref() {
                        Some(player) => player,
                        None => return,
                    };

                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Priest,
                        &PriestReplyServerPacket {
                            reply_code: PriestReply::DoYou,
                        },
                    );
                    WeddingState::WaitingForPartner
                }
                WeddingState::WaitingForPartner | WeddingState::WaitingForPlayer => {
                    self.npc_chat(npc_index, &LANG.wedding_error);
                    self.wedding = None;
                    self.wedding_ticks = 0;
                    return;
                }
                WeddingState::PartnerAgrees => {
                    self.player_chat(partner_id, &LANG.wedding_i_do);
                    WeddingState::PriestDoYouPlayer
                }
                WeddingState::PriestDoYouPlayer => {
                    self.npc_chat(
                        npc_index,
                        &get_lang_string!(
                            &LANG.wedding_do_you,
                            name = partner.name,
                            partner = character.name
                        ),
                    );
                    WeddingState::AskPlayer
                }
                WeddingState::AskPlayer => {
                    let player = match character.player.as_ref() {
                        Some(player) => player,
                        None => return,
                    };

                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Priest,
                        &PriestReplyServerPacket {
                            reply_code: PriestReply::DoYou,
                        },
                    );
                    WeddingState::WaitingForPlayer
                }
                WeddingState::PlayerAgrees => {
                    self.player_chat(player_id, &LANG.wedding_i_do);
                    WeddingState::PriestDialog3
                }
                WeddingState::PriestDialog3 => {
                    self.npc_chat(npc_index, &LANG.wedding_three);
                    let partner_name = partner.name.to_owned();
                    let character_name = character.name.to_owned();

                    self.give_item(player_id, SETTINGS.marriage.ring_item_id, 1);
                    self.give_item(partner_id, SETTINGS.marriage.ring_item_id, 1);
                    self.set_partner(player_id, partner_name);
                    self.set_partner(partner_id, character_name);
                    WeddingState::PriestDialog4
                }
                WeddingState::PriestDialog4 => {
                    self.npc_chat(npc_index, &LANG.wedding_four);
                    WeddingState::Hearts
                }
                WeddingState::Hearts => {
                    self.effect_on_players(
                        &[player_id, partner_id],
                        SETTINGS.marriage.celebration_effect_id,
                    );
                    WeddingState::PriestDialog5AndConfetti
                }
                WeddingState::PriestDialog5AndConfetti => {
                    self.npc_chat(
                        npc_index,
                        &get_lang_string!(
                            &LANG.wedding_five,
                            partner = partner.name,
                            name = character.name
                        ),
                    );

                    self.effect_on_players(&[player_id, partner_id], 11);

                    WeddingState::Done
                }
                WeddingState::Done => {
                    self.npc_chat(npc_index, &LANG.wedding_end);
                    self.wedding = None;
                    self.wedding_ticks = 0;
                    return;
                }
                _ => return,
            };

            if let Some(wedding) = self.wedding.as_mut() {
                self.wedding_ticks = 0;
                wedding.state = next_state;
            }
        }
    }

    pub fn set_partner(&mut self, player_id: i32, name: String) {
        if let Some(character) = self.characters.get_mut(&player_id) {
            character.partner = Some(name);
            character.fiance = None;
        }
    }
}
