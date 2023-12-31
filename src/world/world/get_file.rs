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

use crate::{errors::WrongSessionIdError, CLASS_DB, ITEM_DB, NPC_DB, SPELL_DB};

use super::World;

impl World {
    pub async fn get_file(
        &self,
        player_id: i32,
        file_type: FileType,
        session_id: i32,
        _file_id: Option<i32>,
        warp: bool,
    ) {
        let player = match self.players.get(&player_id) {
            Some(player) => player,
            None => return,
        };

        let actual_session_id = match player.get_session_id().await {
            Ok(session_id) => session_id,
            Err(e) => {
                player.close(format!("Error getting session id: {}", e));
                return;
            }
        };

        if actual_session_id != session_id {
            player.close(format!(
                "{}",
                WrongSessionIdError::new(actual_session_id, session_id,)
            ));
            return;
        }

        let reply: InitInitServerPacket = match file_type {
            FileType::Emf => {
                let map_id = match player.get_map_id().await {
                    Ok(map_id) => map_id,
                    Err(_) => {
                        player.close("Player requested map with no character selected".to_string());
                        return;
                    }
                };

                let mut reply = InitInitServerPacket::new();
                let maps = self.maps.as_ref().expect("Maps not loaded");
                let map = match maps.get(&map_id) {
                    Some(map) => map,
                    None => {
                        player.close(format!("Requested map not found: {}", map_id));
                        return;
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
            _ => return,
        };

        let mut writer = EoWriter::new();

        if let Err(e) = reply.serialize(&mut writer) {
            error!("Failed to serialize InitInitServerPacket: {}", e);
            return;
        }

        player.send(
            PacketAction::Init,
            PacketFamily::Init,
            writer.to_byte_array(),
        );
    }
}
