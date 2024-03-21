use eolib::protocol::{
    net::{
        server::{PriestReply, PriestReplyServerPacket, PriestRequestServerPacket},
        PacketAction, PacketFamily,
    },
    r#pub::NpcType,
};

use crate::{
    map::{Wedding, WeddingState},
    utils::dressed_for_wedding,
    NPC_DB,
};

use super::super::Map;

impl Map {
    pub fn request_wedding(&mut self, player_id: i32, npc_index: i32, name: String) {
        match self.npcs.get(&npc_index) {
            Some(npc) => {
                let npc_data = match NPC_DB.npcs.get(npc.id as usize - 1) {
                    Some(npc_data) => npc_data,
                    None => return,
                };

                if npc_data.r#type != NpcType::Priest {
                    return;
                }
            }
            None => return,
        }

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let player = match character.player {
            Some(ref player) => player.to_owned(),
            None => return,
        };

        let fiance = match self.characters.iter().find(|(_, c)| c.name == name) {
            Some((_, fiance)) => fiance,
            None => {
                player.send(
                    PacketAction::Reply,
                    PacketFamily::Priest,
                    &PriestReplyServerPacket {
                        reply_code: PriestReply::PartnerNotPresent,
                    },
                );
                return;
            }
        };

        match &character.fiance {
            Some(fiance) => {
                if *fiance != name {
                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Priest,
                        &PriestReplyServerPacket {
                            reply_code: PriestReply::NoPermission,
                        },
                    );
                    return;
                }
            }
            None => return,
        }

        if fiance.partner.is_some() {
            player.send(
                PacketAction::Reply,
                PacketFamily::Priest,
                &PriestReplyServerPacket {
                    reply_code: PriestReply::PartnerAlreadyMarried,
                },
            );
            return;
        }

        let fiances_fiance = match fiance.fiance {
            Some(ref fiance_fiance) => fiance_fiance.to_owned(),
            None => String::new(),
        };

        if fiances_fiance != character.name {
            player.send(
                PacketAction::Reply,
                PacketFamily::Priest,
                &PriestReplyServerPacket {
                    reply_code: PriestReply::NoPermission,
                },
            );
            return;
        }

        if !dressed_for_wedding(fiance) {
            player.send(
                PacketAction::Reply,
                PacketFamily::Priest,
                &PriestReplyServerPacket {
                    reply_code: PriestReply::PartnerNotDressed,
                },
            );
            return;
        }

        let player = match fiance.player {
            Some(ref player) => player.to_owned(),
            None => return,
        };

        let partner_name = character.name.to_owned();

        self.wedding = Some(Wedding {
            player_id,
            partner_id: fiance.player_id.unwrap(),
            npc_index,
            state: WeddingState::Requested,
        });

        tokio::spawn(async move {
            let session_id = match player.generate_session_id().await {
                Ok(session_id) => session_id,
                Err(e) => {
                    error!("Error generating session id: {}", e);
                    return;
                }
            };

            player.send(
                PacketAction::Request,
                PacketFamily::Priest,
                &PriestRequestServerPacket {
                    session_id,
                    partner_name,
                },
            );
        });
    }
}
