use eolib::{
    data::SHORT_MAX,
    protocol::{
        Coords,
        net::{
            PacketAction, PacketFamily,
            server::{
                WarpEffect, WarpRequestServerPacket, WarpRequestServerPacketWarpTypeData,
                WarpRequestServerPacketWarpTypeDataMapSwitch, WarpType,
            },
        },
    },
};
use rand::RngExt;

use crate::player::WarpSession;

use super::Player;

impl Player {
    pub async fn request_warp(
        &mut self,
        map_id: i32,
        coords: Coords,
        local: bool,
        animation: Option<WarpEffect>,
    ) {
        let session_id = {
            let mut rng = rand::rng();
            let session_id = rng.random_range(10..SHORT_MAX) as i32;
            self.session_id = Some(session_id);
            session_id
        };

        let warp_session = WarpSession {
            map_id,
            coords,
            local,
            animation,
        };

        self.warp_session = Some(warp_session);

        let _ = self
            .bus
            .send(
                PacketAction::Request,
                PacketFamily::Warp,
                if local {
                    WarpRequestServerPacket {
                        warp_type: WarpType::Local,
                        map_id,
                        session_id,
                        warp_type_data: None,
                    }
                } else {
                    match self.world.get_map(map_id).await {
                        Ok(map) => {
                            let (map_rid, map_file_size) = map
                                .get_rid_and_size()
                                .await
                                .expect("Failed to get rid and size. Timeout");
                            WarpRequestServerPacket {
                                warp_type: WarpType::MapSwitch,
                                map_id,
                                session_id,
                                warp_type_data: Some(
                                    WarpRequestServerPacketWarpTypeData::MapSwitch(
                                        WarpRequestServerPacketWarpTypeDataMapSwitch {
                                            map_rid,
                                            map_file_size,
                                        },
                                    ),
                                ),
                            }
                        }
                        Err(err) => {
                            tracing::warn!("{:?}", err);
                            return;
                        }
                    }
                },
            )
            .await;
    }
}
