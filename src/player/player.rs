use std::{cell::RefCell, collections::VecDeque};

use bytes::Bytes;
use eo::{
    data::{EOChar, EOInt, EOShort, StreamBuilder, MAX2},
    net::PacketProcessor,
    protocol::{PacketAction, PacketFamily},
};
use mysql_async::Pool;
use rand::Rng;
use tokio::{net::TcpStream, sync::mpsc::UnboundedReceiver};

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError},
    map::MapHandle,
    world::WorldHandle,
};

use super::{packet_bus::PacketBus, ClientState, Command, WarpSession};

pub struct Player {
    pub id: EOShort,
    pub rx: UnboundedReceiver<Command>,
    pub queue: RefCell<VecDeque<Bytes>>,
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
    interact_npc_index: Option<EOChar>,
    interact_player_id: Option<EOShort>,
    board_id: Option<EOShort>,
    chest_index: Option<usize>,
    warp_session: Option<WarpSession>,
    trading: bool,
    sleep_cost: Option<EOInt>,
}

mod accept_warp;
mod cancel_trade;
mod close;
mod die;
mod request_warp;

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
            interact_npc_index: None,
            interact_player_id: None,
            board_id: None,
            chest_index: None,
            trading: false,
            sleep_cost: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::AcceptWarp { map_id, session_id } => {
                self.accept_warp(map_id, session_id).await
            }
            Command::CancelTrade { player_id } => self.cancel_trade(player_id).await,
            Command::Close(reason) => {
                self.close(reason).await;
                return false;
            }
            Command::Die => self.die().await,
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
                    let _ = respond_to.send(Err(InvalidStateError::new(
                        ClientState::LoggedIn,
                        self.state,
                    )));
                }
            }
            Command::GetBoardId { respond_to } => {
                let _ = respond_to.send(self.board_id);
            }
            Command::GetCharacter { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(Box::new(character.to_owned())));
                } else if let Some(map) = self.map.as_ref() {
                    if let Some(character) = map.get_character(self.id).await {
                        let _ = respond_to.send(Ok(character));
                    }
                } else {
                    let _ = respond_to.send(Err(InvalidStateError::new(
                        ClientState::Playing,
                        self.state,
                    )));
                }
            }
            Command::GetChestIndex { respond_to } => {
                let _ = respond_to.send(self.chest_index);
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
                    let _ = respond_to.send(Err(InvalidStateError::new(
                        ClientState::Playing,
                        self.state,
                    )));
                }
            }
            Command::GetMapId { respond_to } => {
                if let Some(warp_session) = &self.warp_session {
                    let _ = respond_to.send(Ok(warp_session.map_id));
                } else if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(character.map_id));
                } else {
                    let _ = respond_to.send(Err(InvalidStateError::new(
                        ClientState::Playing,
                        self.state,
                    )));
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
            Command::GetInteractNpcIndex { respond_to } => {
                let _ = respond_to.send(self.interact_npc_index);
            }
            Command::GetInteractPlayerId { respond_to } => {
                let _ = respond_to.send(self.interact_player_id);
            }
            Command::GetSequenceStart { respond_to } => {
                let _ = respond_to.send(self.bus.sequencer.get_sequence_start());
            }
            Command::GetSleepCost { respond_to } => {
                let _ = respond_to.send(self.sleep_cost);
            }
            Command::GetState { respond_to } => {
                let _ = respond_to.send(self.state);
            }
            Command::IsTrading { respond_to } => {
                let _ = respond_to.send(self.trading);
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
            } => self.request_warp(map_id, coords, local, animation).await,
            Command::Send(action, family, data) => {
                let _ = self.bus.send(action, family, data).await;
            }
            Command::SetAccountId(account_id) => {
                self.account_id = account_id;
            }
            Command::SetBoardId(board_id) => {
                self.board_id = Some(board_id);
            }
            Command::SetBusy(busy) => {
                self.busy = busy;
            }
            Command::SetCharacter(character) => {
                self.character = Some(*character);
            }
            Command::SetChestIndex(index) => {
                self.chest_index = Some(index);
            }
            Command::SetInteractNpcIndex(index) => {
                self.interact_npc_index = Some(index);
            }
            Command::SetInteractPlayerId(id) => {
                self.interact_player_id = id;
            }
            Command::SetMap(map) => {
                self.map = Some(map);
            }
            Command::SetSleepCost(cost) => {
                self.sleep_cost = Some(cost);
            }
            Command::SetState(state) => {
                self.state = state;
            }
            Command::SetTrading(trading) => {
                self.trading = trading;
            }
            Command::TakeCharacter { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(Box::new(character.to_owned())));
                    self.character = None;
                } else {
                    let _ = respond_to.send(Err(InvalidStateError::new(
                        ClientState::Playing,
                        self.state,
                    )));
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
