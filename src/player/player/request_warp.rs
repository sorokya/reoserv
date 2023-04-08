use eo::{
    data::{EOShort, Serializeable, StreamBuilder, MAX2},
    protocol::{server::warp, Coords, PacketAction, PacketFamily, WarpAnimation, WarpType},
};
use rand::Rng;

use crate::player::WarpSession;

use super::Player;

impl Player {
    pub async fn request_warp(
        &mut self,
        map_id: EOShort,
        coords: Coords,
        local: bool,
        animation: Option<WarpAnimation>,
    ) {
        let session_id = {
            let mut rng = rand::thread_rng();
            let session_id = rng.gen_range(10..MAX2) as EOShort;
            self.session_id = Some(session_id);
            session_id
        };
        let warp_session = WarpSession {
            map_id,
            coords,
            local,
            animation,
        };

        let request = if local {
            warp::Request {
                warp_type: WarpType::Local,
                map_id,
                session_id,
                data: warp::RequestData::None,
            }
        } else {
            match self.world.get_map(map_id).await {
                Ok(map) => {
                    let (map_rid, map_filesize) = map.get_rid_and_size().await;
                    warp::Request {
                        warp_type: WarpType::MapSwitch,
                        map_id,
                        session_id,
                        data: warp::RequestData::MapSwitch(warp::RequestMapSwitch {
                            map_rid,
                            map_filesize,
                        }),
                    }
                }
                Err(err) => {
                    warn!("{:?}", err);
                    return;
                }
            }
        };

        self.warp_session = Some(warp_session);
        debug!("Send: {:?}", request);

        let mut builder = StreamBuilder::new();
        request.serialize(&mut builder);

        let _ = self
            .bus
            .send(PacketAction::Request, PacketFamily::Warp, builder.get())
            .await;
    }
}
