use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::net::{
        client::FileType,
        server::{
            InitInitServerPacket, InitInitServerPacketReplyCodeData,
            InitInitServerPacketReplyCodeDataFileEcf, InitInitServerPacketReplyCodeDataFileEif,
            InitInitServerPacketReplyCodeDataFileEmf, InitInitServerPacketReplyCodeDataFileEnf,
            InitInitServerPacketReplyCodeDataFileEsf, InitInitServerPacketReplyCodeDataWarpMap,
            InitReply, MapFile, PubFile,
        },
        PacketAction, PacketFamily,
    },
};

use crate::{
    errors::WrongSessionIdError, player::ClientState, CLASS_DB, ITEM_DB, NPC_DB, SPELL_DB,
};

use super::Player;

impl Player {
    pub async fn get_file(
        &mut self,
        file_type: FileType,
        session_id: i32,
        _file_id: Option<i32>,
        warp: bool,
    ) -> bool {
        if warp && self.state != ClientState::InGame {
            return true;
        }

        if !warp && self.state != ClientState::EnteringGame {
            return true;
        }

        let actual_session_id = match self.session_id {
            Some(session_id) => session_id,
            None => {
                self.close("No session id".to_string()).await;
                return false;
            }
        };

        if actual_session_id != session_id {
            self.close(format!(
                "{}",
                WrongSessionIdError::new(actual_session_id, session_id,)
            ))
            .await;
            return false;
        }

        let reply: InitInitServerPacket = match file_type {
            FileType::Emf => {
                let map_id = if warp {
                    match self.warp_session {
                        Some(ref warp_session) => warp_session.map_id,
                        None => {
                            self.close("Player requested map with no warp session".to_string())
                                .await;
                            return false;
                        }
                    }
                } else {
                    match self.character {
                        Some(ref character) => character.map_id,
                        None => {
                            self.close(
                                "Player requested map with no character selected".to_string(),
                            )
                            .await;
                            return false;
                        }
                    }
                };

                let mut reply = InitInitServerPacket::new();
                let map = match self.world.get_map(map_id).await {
                    Ok(map) => map,
                    Err(e) => {
                        self.close(format!("Requested map {} not found: {}", map_id, e))
                            .await;
                        return false;
                    }
                };

                reply.reply_code = if warp {
                    InitReply::WarpMap
                } else {
                    InitReply::FileEmf
                };

                reply.reply_code_data = Some(if warp {
                    InitInitServerPacketReplyCodeData::WarpMap(
                        InitInitServerPacketReplyCodeDataWarpMap {
                            map_file: MapFile {
                                content: map.serialize().await.to_vec(),
                            },
                        },
                    )
                } else {
                    InitInitServerPacketReplyCodeData::FileEmf(
                        InitInitServerPacketReplyCodeDataFileEmf {
                            map_file: MapFile {
                                content: map.serialize().await.to_vec(),
                            },
                        },
                    )
                });
                reply
            }
            FileType::Eif => {
                let mut writer = EoWriter::new();
                ITEM_DB.serialize(&mut writer).unwrap();
                InitInitServerPacket {
                    reply_code: InitReply::FileEif,
                    reply_code_data: Some(InitInitServerPacketReplyCodeData::FileEif(
                        InitInitServerPacketReplyCodeDataFileEif {
                            pub_file: PubFile {
                                file_id: 1, // TODO: Pub splitting
                                content: writer.to_byte_array().to_vec(),
                            },
                        },
                    )),
                }
            }
            FileType::Enf => {
                let mut writer = EoWriter::new();
                NPC_DB.serialize(&mut writer).unwrap();
                InitInitServerPacket {
                    reply_code: InitReply::FileEnf,
                    reply_code_data: Some(InitInitServerPacketReplyCodeData::FileEnf(
                        InitInitServerPacketReplyCodeDataFileEnf {
                            pub_file: PubFile {
                                file_id: 1, // TODO: Pub splitting
                                content: writer.to_byte_array().to_vec(),
                            },
                        },
                    )),
                }
            }
            FileType::Esf => {
                let mut writer = EoWriter::new();
                SPELL_DB.serialize(&mut writer).unwrap();
                InitInitServerPacket {
                    reply_code: InitReply::FileEsf,
                    reply_code_data: Some(InitInitServerPacketReplyCodeData::FileEsf(
                        InitInitServerPacketReplyCodeDataFileEsf {
                            pub_file: PubFile {
                                file_id: 1, // TODO: Pub splitting
                                content: writer.to_byte_array().to_vec(),
                            },
                        },
                    )),
                }
            }
            FileType::Ecf => {
                let mut writer = EoWriter::new();
                CLASS_DB.serialize(&mut writer).unwrap();
                InitInitServerPacket {
                    reply_code: InitReply::FileEcf,
                    reply_code_data: Some(InitInitServerPacketReplyCodeData::FileEcf(
                        InitInitServerPacketReplyCodeDataFileEcf {
                            pub_file: PubFile {
                                file_id: 1, // TODO: Pub splitting
                                content: writer.to_byte_array().to_vec(),
                            },
                        },
                    )),
                }
            }
            _ => return false,
        };

        let _ = self
            .bus
            .send(PacketAction::Init, PacketFamily::Init, reply)
            .await;

        true
    }
}
