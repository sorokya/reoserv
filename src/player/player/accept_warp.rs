use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{
                SitState, WarpAgreeServerPacket, WarpAgreeServerPacketWarpTypeData,
                WarpAgreeServerPacketWarpTypeDataMapSwitch, WarpEffect, WarpType,
            },
            PacketAction, PacketFamily,
        },
        Coords,
    },
};

use crate::{
    errors::{MissingSessionIdError, WrongSessionIdError},
    SETTINGS,
};

use super::Player;

impl Player {
    pub async fn accept_warp(&mut self, map_id: i32, session_id: i32) {
        let warp_session = match &self.warp_session {
            Some(warp_session) => warp_session,
            None => {
                error!("Warp error: {}", MissingSessionIdError);
                return;
            }
        };

        let actual_session_id = match self.session_id {
            Some(session_id) => session_id,
            None => return,
        };

        if actual_session_id != session_id {
            error!(
                "Warp error: {}",
                WrongSessionIdError::new(actual_session_id, session_id,)
            );
            return;
        }

        if let Some(current_map) = &self.map {
            let character = current_map
                .leave(self.id, warp_session.animation, self.interact_player_id)
                .await;
            self.interact_npc_index = None;
            self.chest_index = None;
            self.board_id = None;
            self.character = Some(character);
            self.trading = false;
            self.trade_accepted = false;
            self.interact_player_id = None;
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
            WarpAgreeServerPacket {
                warp_type: WarpType::Local,
                warp_type_data: None,
                nearby: nearby_info,
            }
        } else if let Ok(new_map) = self.world.get_map(map_id).await {
            new_map
                .enter(Box::new(character), warp_session.animation)
                .await;
            let nearby_info = new_map.get_nearby_info(self.id).await;
            self.map = Some(new_map);

            WarpAgreeServerPacket {
                warp_type: WarpType::MapSwitch,
                warp_type_data: Some(WarpAgreeServerPacketWarpTypeData::MapSwitch(
                    WarpAgreeServerPacketWarpTypeDataMapSwitch {
                        map_id,
                        warp_effect: warp_session.animation.unwrap_or(WarpEffect::None),
                    },
                )),
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

            WarpAgreeServerPacket {
                warp_type: WarpType::MapSwitch,
                warp_type_data: Some(WarpAgreeServerPacketWarpTypeData::MapSwitch(
                    WarpAgreeServerPacketWarpTypeDataMapSwitch {
                        map_id: SETTINGS.rescue.map,
                        warp_effect: warp_session.animation.unwrap_or(WarpEffect::None),
                    },
                )),
                nearby: nearby_info,
            }
        };

        let mut writer = EoWriter::new();

        if let Err(e) = agree.serialize(&mut writer) {
            error!("Failed to serialize WarpAgreeServerPacket: {}", e);
            return;
        }

        let _ = self
            .bus
            .send(
                PacketAction::Agree,
                PacketFamily::Warp,
                writer.to_byte_array(),
            )
            .await;
    }
}
