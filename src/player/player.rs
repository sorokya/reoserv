use std::{cell::RefCell, collections::VecDeque};

use bytes::Bytes;
use chrono::{DateTime, Utc};
use mysql_async::Pool;
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
    pub state: ClientState,
    ip: String,
    pub connected_at: DateTime<Utc>,
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
}

mod accept_warp;
mod account;
mod arena_die;
mod begin_handshake;
mod cancel_trade;
mod close;
mod complete_handshake;
mod die;
mod enter_game;
mod generate_session_id;
mod get_ban_duration;
mod get_file;
mod get_welcome_request_data;
#[macro_use]
mod guild;
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
            ping_ticks: 0,
            guild_create_members: Vec::new(),
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::AcceptGuildJoinRequest { player_id } => {
                self.accept_guild_join_request(player_id).await
            }
            Command::AcceptWarp { map_id, session_id } => {
                self.accept_warp(map_id, session_id).await
            }
            Command::AddGuildCreationPlayer { player_id, name } => {
                self.add_guild_creation_player(player_id, name).await;
            }
            Command::ArenaDie { spawn_coords } => self.arena_die(spawn_coords).await,
            Command::BeginHandshake {
                challenge,
                hdid,
                version,
            } => return self.begin_handshake(challenge, hdid, version).await,
            Command::CancelTrade => self.cancel_trade().await,
            Command::ChangePassword {
                username,
                old_password,
                new_password,
            } => {
                return self
                    .change_password(username, old_password, new_password)
                    .await
            }
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
            Command::CreateAccount(packet) => return self.create_account(packet).await,
            Command::CreateCharacter(packet) => return self.create_character(packet).await,
            Command::CreateGuild {
                session_id,
                guild_name,
                guild_tag,
                guild_description,
            } => {
                self.create_guild(session_id, guild_name, guild_tag, guild_description)
                    .await
            }
            Command::DeleteCharacter {
                session_id,
                character_id,
            } => return self.delete_character(session_id, character_id).await,
            Command::Die => self.die().await,
            Command::DisbandGuild { session_id } => self.disband_guild(session_id).await,
            Command::EnterGame { session_id } => return self.enter_game(session_id).await,
            Command::GenerateSessionId { respond_to } => {
                let _ = respond_to.send(self.generate_session_id());
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
            Command::GetFile {
                file_type,
                session_id,
                file_id,
                warp,
            } => return self.get_file(file_type, session_id, file_id, warp).await,
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
            Command::KickGuildMember {
                session_id,
                member_name,
            } => self.kick_guild_member(session_id, member_name).await,
            Command::LeaveGuild { session_id } => self.leave_guild(session_id).await,
            Command::Login { username, password } => return self.login(username, password).await,
            Command::Pong => {
                self.bus.need_pong = false;
            }
            Command::PongNewSequence { respond_to } => {
                self.bus
                    .sequencer
                    .set_start(self.bus.upcoming_sequence_start);
                let _ = respond_to.send(());
            }
            Command::QuestAction { action, args } => self.quest_action(action, args).await,
            Command::RequestAccountCreation { username } => {
                return self.request_account_creation(username).await
            }
            Command::RequestCharacterCreation => return self.request_character_creation().await,
            Command::RequestCharacterDeletion { character_id } => {
                return self.request_character_deletion(character_id).await
            }
            Command::RequestGuildCreation {
                session_id,
                guild_name,
                guild_tag,
            } => {
                self.request_guild_creation(session_id, guild_name, guild_tag)
                    .await
            }
            Command::RequestGuildDetails {
                session_id,
                guild_identity,
            } => self.request_guild_details(session_id, guild_identity).await,
            Command::RequestGuildMemberlist {
                session_id,
                guild_identity,
            } => {
                self.request_guild_memberlist(session_id, guild_identity)
                    .await
            }
            Command::RequestGuildInfo {
                session_id,
                info_type,
            } => self.request_guild_info(session_id, info_type).await,
            Command::RequestWarp {
                map_id,
                coords,
                local,
                animation,
            } => self.request_warp(map_id, coords, local, animation).await,
            Command::SelectCharacter {
                player_handle,
                character_id,
            } => return self.select_character(player_handle, character_id).await,
            Command::Send(action, family, data) => {
                let _ = self.bus.send_buf(action, family, data).await;
            }
            Command::SetBoardId(board_id) => {
                self.board_id = Some(board_id);
            }
            Command::SetBusy(busy) => {
                self.busy = busy;
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
            Command::UpdateGuild {
                session_id,
                info_type_data,
            } => self.update_guild(session_id, info_type_data).await,
            Command::AssignGuildRank {
                session_id,
                member_name,
                rank,
            } => self.assign_guild_rank(session_id, member_name, rank).await,
        }

        true
    }
}
