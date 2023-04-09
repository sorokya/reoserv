use eo::{
    data::{EOShort, Serializeable, StreamBuilder},
    protocol::{server::warp, Coords, PacketAction, PacketFamily, WarpType, SitState},
};

use crate::{
    errors::{MissingSessionIdError, WrongSessionIdError},
    SETTINGS,
};

use super::Player;

impl Player {
    pub async fn accept_warp(&mut self, map_id: EOShort, session_id: EOShort) {
        if let Some(warp_session) = &self.warp_session {
            if let Some(actual_session_id) = self.session_id {
                if actual_session_id != session_id {
                    error!(
                        "Warp error: {}",
                        WrongSessionIdError::new(actual_session_id, session_id,)
                    );
                    return;
                }

                if let Some(current_map) = &self.map {
                    let character = current_map.leave(self.id, warp_session.animation).await;
                    self.character = Some(character);
                }

                let mut character = match &self.character {
                    Some(character) => character.to_owned(),
                    None => {
                        error!("Warp error: expected character to be Some!");
                        return;
                    }
                };

                character.map_id = warp_session.map_id;
                character.coords = warp_session.coords;
                character.sit_state = SitState::Stand;

                self.character = None;

                let agree = if warp_session.local {
                    let current_map = match self.map.as_ref() {
                        Some(current_map) => current_map,
                        None => {
                            error!("Warp error: expected map to be Some!");
                            return;
                        }
                    };

                    current_map
                        .enter(Box::new(character), warp_session.animation)
                        .await;
                    let nearby_info = current_map.get_nearby_info(self.id).await;
                    warp::Agree {
                        warp_type: WarpType::Local,
                        data: warp::AgreeData::None,
                        nearby: nearby_info,
                    }
                } else if let Ok(new_map) = self.world.get_map(map_id).await {
                    new_map
                        .enter(Box::new(character), warp_session.animation)
                        .await;
                    let nearby_info = new_map.get_nearby_info(self.id).await;
                    self.map = Some(new_map);

                    warp::Agree {
                        warp_type: WarpType::MapSwitch,
                        data: warp::AgreeData::MapSwitch(warp::AgreeMapSwitch {
                            map_id,
                            warp_anim: warp_session
                                .animation
                                .unwrap_or(eo::protocol::WarpAnimation::None),
                        }),
                        nearby: nearby_info,
                    }
                } else {
                    character.map_id = SETTINGS.rescue.map;
                    character.coords = Coords {
                        x: SETTINGS.rescue.x,
                        y: SETTINGS.rescue.y,
                    };

                    let map = match self.world.get_map(SETTINGS.rescue.map).await {
                        Ok(map) => map,
                        Err(err) => {
                            error!("Rescue map not found! {}", err);
                            return;
                        }
                    };

                    map.enter(Box::new(character), warp_session.animation).await;
                    let nearby_info = map.get_nearby_info(self.id).await;
                    self.map = Some(map);

                    warp::Agree {
                        warp_type: WarpType::MapSwitch,
                        data: warp::AgreeData::MapSwitch(warp::AgreeMapSwitch {
                            map_id: SETTINGS.rescue.map,
                            warp_anim: warp_session
                                .animation
                                .unwrap_or(eo::protocol::WarpAnimation::None),
                        }),
                        nearby: nearby_info,
                    }
                };

                debug!("Send: {:?}", agree);

                let mut builder = StreamBuilder::new();
                agree.serialize(&mut builder);

                let _ = self
                    .bus
                    .send(PacketAction::Agree, PacketFamily::Warp, builder.get())
                    .await;
            }
        } else {
            error!("Warp error: {}", MissingSessionIdError);
        }
    }
}
