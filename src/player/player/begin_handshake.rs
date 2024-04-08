use eolib::{
    encrypt::{generate_swap_multiple, server_verification_hash},
    packet::{generate_sequence_start, get_init_sequence_bytes},
    protocol::net::{
        server::{
            InitBanType, InitInitServerPacket, InitInitServerPacketReplyCodeData,
            InitInitServerPacketReplyCodeDataBanned,
            InitInitServerPacketReplyCodeDataBannedBanTypeData,
            InitInitServerPacketReplyCodeDataBannedBanTypeDataTemporary,
            InitInitServerPacketReplyCodeDataOk, InitReply,
        },
        PacketAction, PacketFamily, Version,
    },
};

use crate::player::ClientState;

use super::Player;

impl Player {
    // TODO: verify version
    pub async fn begin_handshake(
        &mut self,
        challenge: i32,
        _hdid: String,
        _version: Version,
    ) -> bool {
        if let Some(duration) = self.get_ban_duration().await {
            let _ = self
                .bus
                .send(
                    PacketAction::Init,
                    PacketFamily::Init,
                    InitInitServerPacket {
                        reply_code: InitReply::Banned,
                        reply_code_data: Some(InitInitServerPacketReplyCodeData::Banned(
                            if duration > 0 {
                                InitInitServerPacketReplyCodeDataBanned {
                                    ban_type: InitBanType::Temporary,
                                    ban_type_data: Some(
                                        InitInitServerPacketReplyCodeDataBannedBanTypeData::Temporary(
                                            InitInitServerPacketReplyCodeDataBannedBanTypeDataTemporary {
                                                minutes_remaining: duration as u8,
                                            },
                                        ),
                                    ),
                                }
                            } else {
                                InitInitServerPacketReplyCodeDataBanned {
                                    ban_type: InitBanType::Permanent,
                                    ban_type_data: None,
                                }
                            },
                        )),
                    },
                )
                .await;

            self.close("IP Banned".to_string()).await;
            return false;
        }

        let sequence_start = generate_sequence_start();
        let sequence_bytes = get_init_sequence_bytes(sequence_start);
        self.bus.sequencer.set_start(sequence_start);

        let challenge_response = server_verification_hash(challenge);

        self.bus.client_enryption_multiple = generate_swap_multiple();
        self.bus.server_enryption_multiple = generate_swap_multiple();
        self.state = ClientState::Initialized;

        let _ = self
            .bus
            .send(
                PacketAction::Init,
                PacketFamily::Init,
                InitInitServerPacket {
                    reply_code: InitReply::OK,
                    reply_code_data: Some(InitInitServerPacketReplyCodeData::OK(
                        InitInitServerPacketReplyCodeDataOk {
                            seq1: sequence_bytes[0] as u8,
                            seq2: sequence_bytes[1] as u8,
                            server_encryption_multiple: self.bus.server_enryption_multiple,
                            client_encryption_multiple: self.bus.client_enryption_multiple,
                            challenge_response,
                            player_id: self.id,
                        },
                    )),
                },
            )
            .await;

        true
    }
}
