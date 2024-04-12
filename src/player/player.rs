use std::{cell::RefCell, collections::VecDeque};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use eolib::protocol::net::{server::GuildReplyServerPacket, PacketAction, PacketFamily, Version};
use mysql_async::Pool;
use tokio::{net::TcpStream, sync::mpsc::UnboundedReceiver};

use crate::{character::Character, errors::InvalidStateError, map::MapHandle, world::WorldHandle};

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
    pub state: ClientState,
    ip: String,
    pub connected_at: DateTime<Utc>,
    pub closed: bool,
    login_attempts: i32,
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
    ping_ticks: i32,
    guild_create_members: Vec<i32>,
    version: Version,
    email_pin: Option<String>,
}

mod account;
mod arena_die;
mod cancel_trade;
mod close;
mod die;
mod enter_game;
mod generate_session_id;
mod get_ban_duration;
mod get_file;
mod get_welcome_request_data;
mod handlers;
#[macro_use]
mod guild;
mod generate_email_pin;
mod ping;
mod quest_action;
mod request_warp;
mod send_server_message;
mod take_session_id;
mod tick;

impl Player {
    pub fn new(
        id: i32,
        socket: TcpStream,
        connected_at: DateTime<Utc>,
        rx: UnboundedReceiver<Command>,
        world: WorldHandle,
        pool: Pool,
    ) -> Self {
        let ip = socket.peer_addr().unwrap().ip().to_string();
        Self {
            id,
            bus: PacketBus::new(socket),
            connected_at,
            rx,
            world,
            pool,
            queue: RefCell::new(VecDeque::new()),
            map: None,
            busy: false,
            closed: false,
            account_id: 0,
            state: ClientState::Uninitialized,
            login_attempts: 0,
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
            ping_ticks: 0,
            guild_create_members: Vec::new(),
            version: Version::default(),
            email_pin: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::AddGuildCreationPlayer { player_id, name } => {
                self.add_guild_creation_player(player_id, name).await
            }
            Command::ArenaDie { spawn_coords } => self.arena_die(spawn_coords).await,
            Command::CancelTrade => self.cancel_trade().await,
            Command::Close(reason) => {
                self.close(reason).await;
                return false;
            }
            Command::Die => self.die().await,
            Command::GenerateSessionId { respond_to } => {
                let _ = respond_to.send(self.generate_session_id());
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
            Command::GetMap { respond_to } => {
                if let Some(map) = self.map.as_ref() {
                    let _ = respond_to.send(Ok(map.to_owned()));
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
            Command::GetInteractPlayerId { respond_to } => {
                let _ = respond_to.send(self.interact_player_id);
            }
            Command::GetState { respond_to } => {
                let _ = respond_to.send(self.state);
            }
            Command::GetVersion { respond_to } => {
                let _ = respond_to.send(self.version.clone());
            }
            Command::IsTradeAccepted { respond_to } => {
                let _ = respond_to.send(self.trade_accepted);
            }
            Command::IsTrading { respond_to } => {
                let _ = respond_to.send(self.trading);
            }
            Command::QuestAction { action, args } => self.quest_action(action, args).await,
            Command::RequestWarp {
                map_id,
                coords,
                local,
                animation,
            } => self.request_warp(map_id, coords, local, animation).await,
            Command::SendGuildReply(reply_code) => {
                let _ = self
                    .bus
                    .send(
                        PacketAction::Reply,
                        PacketFamily::Guild,
                        GuildReplyServerPacket {
                            reply_code,
                            reply_code_data: None,
                        },
                    )
                    .await;
            }
            Command::SendServerMessage(message) => self.send_server_message(&message).await,
            Command::Send(action, family, data) => {
                let _ = self.bus.send_buf(action, family, data).await;
            }
            Command::SetBoardId(board_id) => {
                self.board_id = Some(board_id);
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
            Command::SetSleepCost(cost) => {
                self.sleep_cost = Some(cost);
            }
            Command::SetTradeAccepted(accepted) => {
                self.trade_accepted = accepted;
            }
            Command::SetTrading(trading) => {
                self.trading = trading;
            }
            Command::Tick => return self.tick().await,
            Command::UpdatePartyHP { hp_percentage } => {
                if self.state == ClientState::InGame {
                    self.world.update_party_hp(self.id, hp_percentage);
                }
            }
        }

        true
    }
}
