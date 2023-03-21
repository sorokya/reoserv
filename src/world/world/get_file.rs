use eo::{
    data::{EOChar, EOShort, Serializeable},
    protocol::{server::init, FileType, InitReply},
};

use crate::{
    errors::{DataNotFoundError, MissingSessionIdError, WrongSessionIdError},
    player::PlayerHandle,
    CLASS_DB, ITEM_DB, NPC_DB, SPELL_DB,
};

use super::World;

impl World {
    pub async fn get_file(
        &self,
        file_type: FileType,
        session_id: EOShort,
        _file_id: Option<EOChar>,
        player: PlayerHandle,
    ) -> Result<init::Init, Box<dyn std::error::Error + Send + Sync>> {
        if let Ok(actual_session_id) = player.get_session_id().await {
            if actual_session_id != session_id {
                return Err(Box::new(WrongSessionIdError::new(
                    actual_session_id,
                    session_id,
                )));
            }

            match file_type {
                FileType::Map => {
                    let map_id = match player.get_map_id().await {
                        Ok(map_id) => map_id,
                        Err(e) => {
                            warn!("Player requested map with no character selected");
                            return Err(e);
                        }
                    };

                    let mut reply = init::Init::default();
                    let maps = self.maps.as_ref().expect("Maps not loaded");
                    let map = match maps.get(&map_id) {
                        Some(map) => map,
                        None => {
                            error!("Requested map not found: {}", map_id);
                            return Err(Box::new(DataNotFoundError::new(
                                "Map".to_string(),
                                map_id,
                            )));
                        }
                    };
                    reply.reply_code = InitReply::FileEmf;
                    reply.data = init::InitData::FileEmf(init::InitFileEmf {
                        content: map.serialize().await,
                    });
                    Ok(reply)
                }
                FileType::Item => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEif,
                        data: init::InitData::FileEif(init::InitFileEif {
                            file_id: 1, // TODO: Pub splitting
                            content: ITEM_DB.serialize(),
                        }),
                    })
                }
                FileType::Npc => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEnf,
                        data: init::InitData::FileEnf(init::InitFileEnf {
                            file_id: 1, // TODO: Pub splitting
                            content: NPC_DB.serialize(),
                        }),
                    })
                }
                FileType::Spell => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEsf,
                        data: init::InitData::FileEsf(init::InitFileEsf {
                            file_id: 1, // TODO: Pub splitting
                            content: SPELL_DB.serialize(),
                        }),
                    })
                }
                FileType::Class => {
                    Ok(init::Init {
                        reply_code: InitReply::FileEcf,
                        data: init::InitData::FileEcf(init::InitFileEcf {
                            file_id: 1, // TODO: Pub splitting
                            content: CLASS_DB.serialize(),
                        }),
                    })
                }
            }
        } else {
            Err(Box::new(MissingSessionIdError))
        }
    }
}
