use eolib::protocol::net::{
    server::{
        PartyReplyCode, PartyReplyServerPacket, PartyReplyServerPacketReplyCodeData,
        PartyReplyServerPacketReplyCodeDataAlreadyInAnotherParty,
        PartyReplyServerPacketReplyCodeDataAlreadyInYourParty, PartyRequestServerPacket,
    },
    PacketAction, PacketFamily, PartyRequestType,
};

use crate::{player::PartyRequest, utils::in_client_range, SETTINGS};

use super::super::Map;

impl Map {
    pub fn party_request(&self, target_player_id: i32, request: PartyRequest) {
        let player_id = match request {
            PartyRequest::Join(id) => id,
            PartyRequest::Invite(id) => id,
            _ => return,
        };

        let character = match self.characters.get(&player_id) {
            Some(character) => character,
            None => return,
        };

        let target_character = match self.characters.get(&target_player_id) {
            Some(character) => character,
            None => return,
        };

        if target_character.hidden
            || target_character.captcha_open
            || !in_client_range(&character.coords, &target_character.coords)
        {
            return;
        }

        let player = match character.player.as_ref() {
            Some(player) => player.to_owned(),
            None => return,
        };

        let target = match target_character.player.as_ref() {
            Some(player) => player.to_owned(),
            None => return,
        };

        let player_name = character.name.to_owned();

        let world = self.world.to_owned();

        tokio::spawn(async move {
            if let Ok(Some(party)) = world.get_player_party(target_player_id).await {
                let reply_code = match request {
                    PartyRequest::Join(_) => {
                        if party.members.contains(&player_id) {
                            Some(PartyReplyCode::AlreadyInYourParty)
                        } else {
                            None
                        }
                    }
                    PartyRequest::Invite(_) => {
                        if party.members.contains(&player_id) {
                            Some(PartyReplyCode::AlreadyInYourParty)
                        } else {
                            Some(PartyReplyCode::AlreadyInAnotherParty)
                        }
                    }
                    _ => None,
                };

                if let Some(reply_code) = reply_code {
                    player.send(
                        PacketAction::Reply,
                        PacketFamily::Party,
                        &PartyReplyServerPacket {
                            reply_code,
                            reply_code_data: match reply_code {
                                PartyReplyCode::AlreadyInAnotherParty => Some(
                                    PartyReplyServerPacketReplyCodeData::AlreadyInAnotherParty(
                                        PartyReplyServerPacketReplyCodeDataAlreadyInAnotherParty {
                                            player_name: player_name.to_owned(),
                                        },
                                    ),
                                ),
                                PartyReplyCode::AlreadyInYourParty => {
                                    Some(PartyReplyServerPacketReplyCodeData::AlreadyInYourParty(
                                        PartyReplyServerPacketReplyCodeDataAlreadyInYourParty {
                                            player_name: player_name.to_owned(),
                                        },
                                    ))
                                }
                                _ => None,
                            },
                        },
                    );

                    return;
                }
            }

            // Check if party is full
            if let Ok(Some(party)) = world
                .get_player_party(match request {
                    PartyRequest::Join(_) => target_player_id,
                    PartyRequest::Invite(_) => player_id,
                    _ => return,
                })
                .await
            {
                if party.members.len() as i32 >= SETTINGS.limits.max_party_size {
                    let packet = PartyReplyServerPacket {
                        reply_code: PartyReplyCode::PartyIsFull,
                        reply_code_data: None,
                    };

                    player.send(PacketAction::Reply, PacketFamily::Party, &packet);

                    return;
                }
            }

            target.set_party_request(request);

            target.send(
                PacketAction::Request,
                PacketFamily::Party,
                &PartyRequestServerPacket {
                    request_type: match request {
                        PartyRequest::Join(_) => PartyRequestType::Join,
                        PartyRequest::Invite(_) => PartyRequestType::Invite,
                        _ => return,
                    },
                    inviter_player_id: player_id,
                    player_name,
                },
            );
        });
    }
}
