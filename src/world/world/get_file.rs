use eo::{
    data::{EOChar, EOShort, Serializeable, StreamBuilder},
    protocol::{server::init, FileType, InitReply, PacketAction, PacketFamily},
};

use crate::{errors::WrongSessionIdError, CLASS_DB, ITEM_DB, NPC_DB, SPELL_DB};

use super::World;

impl World {
    pub async fn get_file(
        &self,
        player_id: EOShort,
        file_type: FileType,
        session_id: EOShort,
        _file_id: Option<EOChar>,
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

        let reply = match file_type {
            FileType::Map => {
                let map_id = match player.get_map_id().await {
                    Ok(map_id) => map_id,
                    Err(_) => {
                        player.close("Player requested map with no character selected".to_string());
                        return;
                    }
                };

                let mut reply = init::Init::default();
                let maps = self.maps.as_ref().expect("Maps not loaded");
                let map = match maps.get(&map_id) {
                    Some(map) => map,
                    None => {
                        player.close(format!("Requested map not found: {}", map_id));
                        return;
                    }
                };
                reply.reply_code = if warp {
                    InitReply::WarpFileEmf
                } else {
                    InitReply::FileEmf
                };
                reply.data = if warp {
                    init::InitData::WarpFileEmf(init::InitWarpFileEmf {
                        content: map.serialize().await.to_vec(),
                    })
                } else {
                    init::InitData::FileEmf(init::InitFileEmf {
                        content: map.serialize().await.to_vec(),
                    })
                };
                reply
            }
            FileType::Item => {
                let mut builder = StreamBuilder::new();
                ITEM_DB.serialize(&mut builder);
                init::Init {
                    reply_code: InitReply::FileEif,
                    data: init::InitData::FileEif(init::InitFileEif {
                        file_id: 1, // TODO: Pub splitting
                        content: builder.get().to_vec(),
                    }),
                }
            }
            FileType::Npc => {
                let mut builder = StreamBuilder::new();
                NPC_DB.serialize(&mut builder);
                init::Init {
                    reply_code: InitReply::FileEnf,
                    data: init::InitData::FileEnf(init::InitFileEnf {
                        file_id: 1, // TODO: Pub splitting
                        content: builder.get().to_vec(),
                    }),
                }
            }
            FileType::Spell => {
                let mut builder = StreamBuilder::new();
                SPELL_DB.serialize(&mut builder);
                init::Init {
                    reply_code: InitReply::FileEsf,
                    data: init::InitData::FileEsf(init::InitFileEsf {
                        file_id: 1, // TODO: Pub splitting
                        content: builder.get().to_vec(),
                    }),
                }
            }
            FileType::Class => {
                let mut builder = StreamBuilder::new();
                CLASS_DB.serialize(&mut builder);
                init::Init {
                    reply_code: InitReply::FileEcf,
                    data: init::InitData::FileEcf(init::InitFileEcf {
                        file_id: 1, // TODO: Pub splitting
                        content: builder.get().to_vec(),
                    }),
                }
            }
        };

        let mut builder = StreamBuilder::new();
        reply.serialize(&mut builder);
        player.send(PacketAction::Init, PacketFamily::Init, builder.get());
    }
}
