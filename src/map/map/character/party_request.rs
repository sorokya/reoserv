use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        server::{
            PartyReplyCode, PartyReplyServerPacket, PartyReplyServerPacketReplyCodeData,
            PartyReplyServerPacketReplyCodeDataAlreadyInAnotherParty,
            PartyReplyServerPacketReplyCodeDataAlreadyInYourParty, PartyRequestServerPacket,
        },
        PacketAction, PacketFamily, PartyRequestType,
    },
};

use crate::{player::PartyRequest, utils::in_client_range, SETTINGS};

use super::super::Map;

impl Map {
    pub async fn party_request(&self, target_player_id: i32, request: PartyRequest) {
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

        if !in_client_range(&character.coords, &target_character.coords) {
            return;
        }

        // Check if player already in a party
        if let Some(party) = self.world.get_player_party(target_player_id).await {
            let mut writer = EoWriter::new();

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
                let packet = PartyReplyServerPacket {
                    reply_code,
                    reply_code_data: match reply_code {
                        PartyReplyCode::AlreadyInAnotherParty => {
                            Some(PartyReplyServerPacketReplyCodeData::AlreadyInAnotherParty(
                                PartyReplyServerPacketReplyCodeDataAlreadyInAnotherParty {
                                    player_name: target_character.name.clone(),
                                },
                            ))
                        }
                        PartyReplyCode::AlreadyInYourParty => {
                            Some(PartyReplyServerPacketReplyCodeData::AlreadyInYourParty(
                                PartyReplyServerPacketReplyCodeDataAlreadyInYourParty {
                                    player_name: target_character.name.clone(),
                                },
                            ))
                        }
                        _ => None,
                    },
                };

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize PartyReplyServerPacket: {}", e);
                    return;
                }

                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Party,
                    writer.to_byte_array(),
                );

                return;
            }
        }

        // Check if party is full
        if let Some(party) = self
            .world
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

                let mut writer = EoWriter::new();

                if let Err(e) = packet.serialize(&mut writer) {
                    error!("Failed to serialize PartyReplyServerPacket: {}", e);
                    return;
                }

                character.player.as_ref().unwrap().send(
                    PacketAction::Reply,
                    PacketFamily::Party,
                    writer.to_byte_array(),
                );

                return;
            }
        }

        let target = match target_character.player.as_ref() {
            Some(player) => player,
            None => return,
        };

        target.set_party_request(request);

        let packet = PartyRequestServerPacket {
            request_type: match request {
                PartyRequest::Join(_) => PartyRequestType::Join,
                PartyRequest::Invite(_) => PartyRequestType::Invite,
                _ => return,
            },
            inviter_player_id: player_id,
            player_name: character.name.clone(),
        };

        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize PartyRequestServerPacket: {}", e);
            return;
        }

        target.send(
            PacketAction::Request,
            PacketFamily::Party,
            writer.to_byte_array(),
        );
    }
}
