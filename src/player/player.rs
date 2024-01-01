use std::{cell::RefCell, collections::VecDeque};

use bytes::Bytes;
use eolib::{
    data::{EoSerialize, EoWriter, SHORT_MAX},
    packet::{generate_sequence_start, get_ping_sequence_bytes},
    protocol::net::{server::ConnectionPlayerServerPacket, PacketAction, PacketFamily},
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

use super::{packet_bus::PacketBus, ClientState, Command, PartyRequest, WarpSession};

pub struct Player {
    pub id: i32,
    pub rx: UnboundedReceiver<Command>,
    pub queue: RefCell<VecDeque<Bytes>>,
    pub bus: PacketBus,
    pub world: WorldHandle,
    // TODO: just use character's map?
    pub map: Option<MapHandle>,
    pub busy: bool,
    account_id: i32,
    pool: Pool,
    state: ClientState,
    ip: String,
    character: Option<Character>,
    session_id: Option<i32>,
    interact_npc_index: Option<i32>,
    interact_player_id: Option<i32>,
    board_id: Option<i32>,
    chest_index: Option<usize>,
    warp_session: Option<WarpSession>,
    trading: bool,
    trade_accepted: bool,
    sleep_cost: Option<i32>,
    party_request: PartyRequest,
}

mod accept_warp;
mod arena_die;
mod begin_handshake;
mod cancel_trade;
mod close;
mod complete_handshake;
mod die;
mod get_ban_duration;
mod request_warp;

impl Player {
    pub fn new(
        id: i32,
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
            trade_accepted: false,
            sleep_cost: None,
            party_request: PartyRequest::None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::AcceptWarp { map_id, session_id } => {
                self.accept_warp(map_id, session_id).await
            }
            Command::BeginHandshake {
                challenge,
                hdid,
                version,
            } => return self.begin_handshake(challenge, hdid, version).await,
            Command::CancelTrade => self.cancel_trade().await,
            Command::Close(reason) => {
                self.close(reason).await;
                return false;
            }
            Command::CompleteHandshake {
                player_id,
                client_encryption_multiple,
                server_encryption_multiple,
            } => {
                return self
                    .complete_handshake(
                        player_id,
                        client_encryption_multiple,
                        server_encryption_multiple,
                    )
                    .await
            }
            Command::ArenaDie { spawn_coords } => self.arena_die(spawn_coords).await,
            Command::Die => self.die().await,
            Command::GenerateSessionId { respond_to } => {
                let mut rng = rand::thread_rng();
                let id = rng.gen_range(1..SHORT_MAX) as i32;
                self.session_id = Some(id);
                let _ = respond_to.send(id);
            }
            Command::GetAccountId { respond_to } => {
                if let ClientState::LoggedIn | ClientState::InGame = self.state {
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
                    let _ = respond_to
                        .send(Err(InvalidStateError::new(ClientState::InGame, self.state)));
                }
            }
            Command::GetChestIndex { respond_to } => {
                let _ = respond_to.send(self.chest_index);
            }
            Command::GetIpAddr { respond_to } => {
                let _ = respond_to.send(self.ip.clone());
            }
            Command::GetMap { respond_to } => {
                if let Some(map) = self.map.as_ref() {
                    let _ = respond_to.send(Ok(map.to_owned()));
                } else {
                    let _ = respond_to
                        .send(Err(InvalidStateError::new(ClientState::InGame, self.state)));
                }
            }
            Command::GetMapId { respond_to } => {
                if let Some(warp_session) = &self.warp_session {
                    let _ = respond_to.send(Ok(warp_session.map_id));
                } else if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(character.map_id));
                } else {
                    let _ = respond_to
                        .send(Err(InvalidStateError::new(ClientState::InGame, self.state)));
                }
            }
            Command::GetPlayerId { respond_to } => {
                let _ = respond_to.send(self.id);
            }
            Command::GetPartyRequest { respond_to } => {
                let _ = respond_to.send(self.party_request);
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
                let _ = respond_to.send(self.bus.sequencer.get_start());
            }
            Command::GetSleepCost { respond_to } => {
                let _ = respond_to.send(self.sleep_cost);
            }
            Command::GetState { respond_to } => {
                let _ = respond_to.send(self.state);
            }
            Command::IsTradeAccepted { respond_to } => {
                let _ = respond_to.send(self.trade_accepted);
            }
            Command::IsTrading { respond_to } => {
                let _ = respond_to.send(self.trading);
            }
            Command::GenSequence { respond_to } => {
                let sequence = self.bus.sequencer.next_sequence();
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
                    self.bus.upcoming_sequence_start = generate_sequence_start();
                    let mut writer = EoWriter::with_capacity(3);
                    let sequence_bytes = get_ping_sequence_bytes(self.bus.upcoming_sequence_start);
                    let packet = ConnectionPlayerServerPacket {
                        seq1: sequence_bytes[0],
                        seq2: sequence_bytes[1],
                    };

                    if let Err(e) = packet.serialize(&mut writer) {
                        error!("Error serializing ConnectionPlayerServerPacket: {}", e);
                        return false;
                    }

                    self.bus.need_pong = true;
                    self.bus
                        .send(
                            PacketAction::Player,
                            PacketFamily::Connection,
                            writer.to_byte_array(),
                        )
                        .await
                        .unwrap();
                }
            }
            Command::Pong => {
                self.bus.need_pong = false;
            }
            Command::PongNewSequence { respond_to } => {
                self.bus
                    .sequencer
                    .set_start(self.bus.upcoming_sequence_start);
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
            Command::SetPartyRequest(request) => {
                self.party_request = request;
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
            Command::SetTradeAccepted(accepted) => {
                self.trade_accepted = accepted;
            }
            Command::SetTrading(trading) => {
                self.trading = trading;
            }
            Command::TakeCharacter { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let _ = respond_to.send(Ok(Box::new(character.to_owned())));
                    self.character = None;
                } else {
                    let _ = respond_to
                        .send(Err(InvalidStateError::new(ClientState::InGame, self.state)));
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
            Command::UpdatePartyHP { hp_percentage } => {
                if self.state == ClientState::InGame {
                    self.world.update_party_hp(self.id, hp_percentage);
                }
            }
        }

        true
    }
}
