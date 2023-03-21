use std::{cell::RefCell, collections::VecDeque};

use eo::{
    data::{EOInt, EOShort, Serializeable, StreamBuilder, MAX2},
    protocol::{server::warp, PacketAction, PacketFamily, WarpType}, net::PacketProcessor,
};
use mysql_async::Pool;
use rand::Rng;
use tokio::{net::TcpStream, sync::mpsc::UnboundedReceiver};

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError, WrongSessionIdError},
    map::MapHandle,
    world::WorldHandle,
    PacketBuf,
};

use super::{packet_bus::PacketBus, Command, WarpSession, ClientState};

pub struct Player {
    pub id: EOShort,
    pub rx: UnboundedReceiver<Command>,
    pub queue: RefCell<VecDeque<PacketBuf>>,
    pub bus: PacketBus,
    pub world: WorldHandle,
    // TODO: just use character's map?
    pub map: Option<MapHandle>,
    pub busy: bool,
    account_id: EOInt,
    pool: Pool,
    state: ClientState,
    ip: String,
    character: Option<Character>,
    session_id: Option<EOShort>,
    warp_session: Option<WarpSession>,
}

impl Player {
    pub fn new(
        id: EOShort,
        socket: TcpStream,
        rx: UnboundedReceiver<Command>,
        world: WorldHandle,
        pool: Pool,
    ) -> Self {
        let ip = socket.peer_addr().unwrap().ip().to_string();
        Self {
            id,
            bus: PacketBus::new(socket),
            rx,
            world,
            pool,
            queue: RefCell::new(VecDeque::new()),
            map: None,
            busy: false,
            account_id: 0,
            state: ClientState::Uninitialized,
            ip,
            character: None,
            warp_session: None,
            session_id: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::AcceptWarp { map_id, session_id } => {
                if let Some(warp_session) = &self.warp_session {
                    if let Some(actual_session_id) = self.session_id {
                        if actual_session_id != session_id {
                            error!(
                                "Warp error: {}",
                                WrongSessionIdError::new(actual_session_id, session_id,)
                            );
                            return true;
                        }

                        if let Some(current_map) = &self.map {
                            let agree = if warp_session.local {
                                let mut character =
                                    current_map.leave(self.id, warp_session.animation).await;
                                character.coords = warp_session.coords;
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
                                let mut character =
                                    current_map.leave(self.id, warp_session.animation).await;
                                character.map_id = warp_session.map_id;
                                character.coords = warp_session.coords;
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
                                warn!("Map not found: {}", map_id);
                                return true;
                            };

                            debug!("Send: {:?}", agree);
                            let _ = self
                                .bus
                                .send(PacketAction::Agree, PacketFamily::Warp, agree.serialize())
                                .await;
                        }
                    } else {
                        error!("Warp error: {}", MissingSessionIdError);
                    }
                } else {
                    error!("Warp error: {}", MissingSessionIdError);
                }
            }
            Command::Close(reason) => {
                self.queue.borrow_mut().clear();
                if let Some(map) = self.map.as_ref() {
                    let mut character = map.leave(self.id, None).await;
                    let pool = self.pool.clone();
                    let _ = tokio::task::Builder::new()
                        .name("character_save")
                        .spawn(async move {
                            let mut conn = pool.get_conn().await.unwrap();
                            if let Some(logged_in_at) = character.logged_in_at {
                                let now = chrono::Utc::now();
                                character.usage +=
                                    (now.timestamp() - logged_in_at.timestamp()) as u32 / 60;
                            }
                            character.save(&mut conn).await.unwrap();
                        });
                }

                let character_name = self
                    .character
                    .as_ref()
                    .map(|c| c.name.clone())
                    .unwrap_or_default();
                self.world
                    .drop_player(self.id, self.account_id, character_name)
                    .await
                    .unwrap();
                info!("player {} connection closed: {:?}", self.id, reason);
                return false;
            }
            Command::GenerateSessionId { respond_to } => {
                let mut rng = rand::thread_rng();
                let id = rng.gen_range(1..MAX2) as EOShort;
                self.session_id = Some(id);
                let _ = respond_to.send(id);
            }
            Command::GetAccountId { respond_to } => {
                if let ClientState::LoggedIn | ClientState::Playing = self.state {
                    let _ = respond_to.send(Ok(self.account_id));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(ClientState::LoggedIn, self.state)));
                }
            }
            Command::GetCharacter { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(Box::new(character.to_owned())));
                } else if let Some(map) = self.map.as_ref() {
                    if let Some(character) = map.get_character(self.id).await {
                        let _ = respond_to.send(Ok(character));
                    }
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(ClientState::Playing, self.state)));
                }
            }
            Command::GenEncodingMultiples { respond_to } => {
                self.bus.packet_processor = PacketProcessor::new();
                respond_to
                    .send([
                        self.bus.packet_processor.encode_multiple,
                        self.bus.packet_processor.decode_multiple,
                    ])
                    .unwrap();
            }
            Command::GetEncodingMultiples { respond_to } => {
                respond_to
                    .send([
                        self.bus.packet_processor.encode_multiple,
                        self.bus.packet_processor.decode_multiple,
                    ])
                    .unwrap();
            }
            Command::GetIpAddr { respond_to } => {
                let _ = respond_to.send(self.ip.clone());
            }
            Command::GetMap { respond_to } => {
                if let Some(map) = self.map.as_ref() {
                    let _ = respond_to.send(Ok(map.to_owned()));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(ClientState::Playing, self.state)));
                }
            }
            Command::GetMapId { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(character.map_id));
                } else if let Some(warp_session) = &self.warp_session {
                    let _ = respond_to.send(Ok(warp_session.map_id));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(ClientState::Playing, self.state)));
                }
            }
            Command::GetPlayerId { respond_to } => {
                let _ = respond_to.send(self.id);
            }
            Command::GetSequenceBytes { respond_to } => {
                respond_to
                    .send(self.bus.sequencer.get_init_sequence_bytes())
                    .unwrap();
            }
            Command::GetSessionId { respond_to } => {
                if let Some(session_id) = self.session_id {
                    let _ = respond_to.send(Ok(session_id));
                } else {
                    let _ = respond_to.send(Err(MissingSessionIdError));
                }
            }
            Command::GetSequenceStart { respond_to } => {
                let _ = respond_to.send(self.bus.sequencer.get_sequence_start());
            }
            Command::GetState { respond_to } => {
                let _ = respond_to.send(self.state);
            }
            Command::GenSequence { respond_to } => {
                let sequence = self.bus.sequencer.gen_sequence();
                let _ = respond_to.send(sequence);
            }
            Command::Ping => {
                if self.state == ClientState::Uninitialized {
                    return true;
                }

                if self.bus.need_pong {
                    info!("player {} connection closed: ping timeout", self.id);
                    return false;
                } else {
                    self.bus.sequencer.ping_new_sequence();
                    let sequence = self.bus.sequencer.get_update_sequence_bytes();
                    let mut builder = StreamBuilder::with_capacity(3);
                    builder.add_short(sequence.0);
                    builder.add_char(sequence.1);
                    self.bus.need_pong = true;
                    self.bus
                        .send(
                            PacketAction::Player,
                            PacketFamily::Connection,
                            builder.get(),
                        )
                        .await
                        .unwrap();
                }
            }
            Command::Pong => {
                self.bus.need_pong = false;
            }
            Command::PongNewSequence { respond_to } => {
                self.bus.sequencer.pong_new_sequence();
                let _ = respond_to.send(());
            }
            Command::RequestWarp {
                map_id,
                coords,
                local,
                animation,
            } => {
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
                            return true;
                        }
                    }
                };

                self.warp_session = Some(warp_session);
                debug!("Send: {:?}", request);
                let _ = self
                    .bus
                    .send(
                        PacketAction::Request,
                        PacketFamily::Warp,
                        request.serialize(),
                    )
                    .await;
            }
            Command::Send(action, family, data) => {
                let _ = self.bus.send(action, family, data).await;
            }
            Command::SetAccountId(account_id) => {
                self.account_id = account_id;
            }
            Command::SetBusy(busy) => {
                self.busy = busy;
            }
            Command::SetCharacter(character) => {
                self.character = Some(*character);
            }
            Command::SetMap(map) => {
                self.map = Some(map);
            }
            Command::SetState(state) => {
                self.state = state;
            }
            Command::TakeCharacter { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(Box::new(character.to_owned())));
                    self.character = None;
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(ClientState::Playing, self.state)));
                }
            }
            Command::TakeSessionId { respond_to } => {
                if let Some(session_id) = self.session_id {
                    self.session_id = None;
                    let _ = respond_to.send(Ok(session_id));
                } else {
                    let _ = respond_to.send(Err(MissingSessionIdError));
                }
            }
        }

        true
    }
}
