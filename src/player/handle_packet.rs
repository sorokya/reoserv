use bytes::Bytes;
use eolib::{
    data::EoReader,
    protocol::net::{PacketAction, PacketFamily},
};

use crate::{deep::FAMILY_CAPTCHA, SETTINGS};

use super::{ClientState, Player};

impl Player {
    pub async fn handle_packet(&mut self, packet: Bytes) {
        let reader = EoReader::new(packet);
        let action = PacketAction::from(reader.get_byte());
        if let PacketAction::Unrecognized(id) = action {
            if id != 0xfe {
                self.close("invalid packet action".to_string()).await;
                return;
            }
        }

        let family = PacketFamily::from(reader.get_byte());
        if let PacketFamily::Unrecognized(id) = family {
            if id != 0xfe && id != FAMILY_CAPTCHA {
                self.close("invalid packet family".to_string()).await;
                return;
            }
        }

        if self.state != ClientState::Uninitialized {
            if family != PacketFamily::Init {
                if family == PacketFamily::Connection && action == PacketAction::Ping {
                    self.bus
                        .sequencer
                        .set_start(self.bus.upcoming_sequence_start);
                }

                let server_sequence = self.bus.sequencer.next_sequence();
                let client_sequence = reader.get_char();

                if SETTINGS.server.enforce_sequence && server_sequence != client_sequence {
                    self.close(format!(
                        "sending invalid sequence: Got {}, expected {}.",
                        client_sequence, server_sequence
                    ))
                    .await;
                    return;
                }
            } else {
                self.bus.sequencer.next_sequence();
            }
        }

        match family {
            PacketFamily::Account => self.handle_account(action, reader).await,
            PacketFamily::AdminInteract => self.handle_admin_interact(action, reader),
            PacketFamily::Attack => self.handle_attack(action, reader),
            PacketFamily::Bank => self.handle_bank(action, reader),
            PacketFamily::Barber => self.handle_barber(action, reader),
            PacketFamily::Board => self.handle_board(action, reader),
            PacketFamily::Book => self.handle_book(action, reader),
            PacketFamily::Chair => self.handle_chair(action, reader),
            PacketFamily::Character => self.handle_character(action, reader).await,
            PacketFamily::Chest => self.handle_chest(action, reader),
            PacketFamily::Citizen => self.handle_citizen(action, reader),
            PacketFamily::Connection => self.handle_connection(action, reader).await,
            PacketFamily::Door => self.handle_door(action, reader),
            PacketFamily::Emote => self.handle_emote(action, reader),
            PacketFamily::Face => self.handle_face(action, reader),
            PacketFamily::Global => {} // no-op
            PacketFamily::Guild => self.handle_guild(action, reader),
            PacketFamily::Init => self.handle_init(action, reader).await,
            PacketFamily::Item => self.handle_item(action, reader),
            PacketFamily::Jukebox => self.handle_jukebox(action, reader),
            PacketFamily::Locker => self.handle_locker(action, reader),
            PacketFamily::Login => self.handle_login(action, reader).await,
            PacketFamily::Marriage => self.handle_marriage(action, reader),
            PacketFamily::Message => self.handle_message(action).await,
            PacketFamily::NpcRange => self.handle_npc_range(action, reader),
            PacketFamily::Paperdoll => self.handle_paperdoll(action, reader),
            PacketFamily::Party => self.handle_party(action, reader),
            PacketFamily::PlayerRange => self.handle_player_range(action, reader),
            PacketFamily::Players => self.handle_players(action, reader),
            PacketFamily::Priest => self.handle_priest(action, reader),
            PacketFamily::Quest => self.handle_quest(action, reader),
            PacketFamily::Range => self.handle_range(action, reader),
            PacketFamily::Refresh => self.handle_refresh(action),
            PacketFamily::Shop => self.handle_shop(action, reader),
            PacketFamily::Sit => self.handle_sit(action, reader),
            PacketFamily::Spell => self.handle_spell(action, reader),
            PacketFamily::StatSkill => self.handle_stat_skill(action, reader),
            PacketFamily::Talk => self.handle_talk(action, reader),
            PacketFamily::Trade => self.handle_trade(action, reader),
            PacketFamily::Walk => self.handle_walk(reader),
            PacketFamily::Warp => self.handle_warp(action, reader).await,
            PacketFamily::Unrecognized(0xfe) => {} // ignored packet
            PacketFamily::Unrecognized(FAMILY_CAPTCHA) => self.handle_captcha(action, reader).await,
            PacketFamily::Welcome => self.handle_welcome(action, reader).await,
            _ => {
                error!("Unhandled packet {:?}_{:?}", action, family);
            }
        }
    }
}
