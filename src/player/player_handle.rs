use std::time::Duration;

use bytes::Bytes;
use chrono::{DateTime, Utc};
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            client::{
                AccountCreateClientPacket, CharacterCreateClientPacket, FileType,
                GuildAgreeClientPacketInfoTypeData, GuildInfoType,
            },
            server::WarpEffect,
            PacketAction, PacketFamily, Version,
        },
        Coords,
    },
};
use mysql_async::Pool;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::{character::Character, map::MapHandle, world::WorldHandle};

use super::{handle_packet::handle_packet, player::Player, ClientState, Command, PartyRequest};

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(
        id: i32,
        socket: TcpStream,
        connected_at: DateTime<Utc>,
        world: WorldHandle,
        pool: Pool,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let player = Player::new(id, socket, connected_at, rx, world, pool);
        let _ = tokio::task::Builder::new()
            .name(&format!("Player {}", id))
            .spawn(run_player(player, PlayerHandle::for_tx(tx.clone())));

        Self { tx }
    }

    fn for_tx(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    pub fn accept_guild_join_request(&self, player_id: i32) {
        let _ = self.tx.send(Command::AcceptGuildJoinRequest { player_id });
    }

    pub fn accept_warp(&self, map_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::AcceptWarp { map_id, session_id });
    }

    pub fn add_guild_creation_player(&self, player_id: i32, name: String) {
        let _ = self
            .tx
            .send(Command::AddGuildCreationPlayer { player_id, name });
    }

    pub fn arena_die(&self, spawn_coords: Coords) {
        let _ = self.tx.send(Command::ArenaDie { spawn_coords });
    }

    pub fn begin_handshake(&self, challenge: i32, hdid: String, version: Version) {
        let _ = self.tx.send(Command::BeginHandshake {
            challenge,
            hdid,
            version,
        });
    }

    pub fn cancel_trade(&self) {
        let _ = self.tx.send(Command::CancelTrade);
    }

    pub fn change_password(&self, username: String, old_password: String, new_password: String) {
        let _ = self.tx.send(Command::ChangePassword {
            username,
            old_password,
            new_password,
        });
    }

    pub fn close(&self, reason: String) {
        let _ = self.tx.send(Command::Close(reason));
    }

    pub fn complete_handshake(
        &self,
        player_id: i32,
        client_encryption_multiple: i32,
        server_encryption_multiple: i32,
    ) {
        let _ = self.tx.send(Command::CompleteHandshake {
            player_id,
            client_encryption_multiple,
            server_encryption_multiple,
        });
    }

    pub fn create_account(&self, packet: AccountCreateClientPacket) {
        let _ = self.tx.send(Command::CreateAccount(packet));
    }

    pub fn create_character(&self, packet: CharacterCreateClientPacket) {
        let _ = self.tx.send(Command::CreateCharacter(packet));
    }

    pub fn create_guild(
        &self,
        session_id: i32,
        guild_name: String,
        guild_tag: String,
        guild_description: String,
    ) {
        let _ = self.tx.send(Command::CreateGuild {
            session_id,
            guild_name,
            guild_tag,
            guild_description,
        });
    }

    pub fn delete_character(&self, session_id: i32, character_id: i32) {
        let _ = self.tx.send(Command::DeleteCharacter {
            session_id,
            character_id,
        });
    }

    pub fn die(&self) {
        let _ = self.tx.send(Command::Die);
    }

    pub fn disband_guild(&self, session_id: i32) {
        let _ = self.tx.send(Command::DisbandGuild { session_id });
    }

    pub fn enter_game(&self, session_id: i32) {
        let _ = self.tx.send(Command::EnterGame { session_id });
    }

    pub async fn generate_session_id(
        &self,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GenerateSessionId { respond_to: tx });
        match rx.await {
            Ok(session_id) => Ok(session_id),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub async fn get_board_id(&self) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetBoardId { respond_to: tx });
        match rx.await {
            Ok(board_id) => board_id,
            Err(_) => None,
        }
    }

    pub async fn get_character(
        &self,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacter { respond_to: tx });
        match rx.await {
            Ok(Ok(character)) => Ok(character),
            Ok(Err(e)) => Err(Box::new(e)),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub async fn get_chest_index(&self) -> Option<usize> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetChestIndex { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn get_file(&self, file_type: FileType, session_id: i32, file_id: Option<i32>, warp: bool) {
        let _ = self.tx.send(Command::GetFile {
            file_type,
            session_id,
            file_id,
            warp,
        });
    }

    pub async fn get_map(&self) -> Result<MapHandle, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMap { respond_to: tx });
        match rx.await {
            Ok(Ok(map)) => Ok(map),
            Ok(Err(e)) => Err(Box::new(e)),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub async fn get_party_request(&self) -> PartyRequest {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPartyRequest { respond_to: tx });
        match rx.await {
            Ok(party_request) => party_request,
            Err(_) => PartyRequest::None,
        }
    }

    pub async fn get_player_id(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerId { respond_to: tx });
        match rx.await {
            Ok(player_id) => Ok(player_id),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub async fn get_session_id(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSessionId { respond_to: tx });
        match rx.await {
            Ok(Ok(session_id)) => Ok(session_id),
            Ok(Err(e)) => Err(Box::new(e)),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub async fn get_interact_npc_index(&self) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetInteractNpcIndex { respond_to: tx });
        match rx.await {
            Ok(index) => index,
            Err(_) => None,
        }
    }

    pub async fn get_interact_player_id(&self) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetInteractPlayerId { respond_to: tx });
        match rx.await {
            Ok(index) => index,
            Err(_) => None,
        }
    }

    pub async fn gen_sequence(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GenSequence { respond_to: tx });
        match rx.await {
            Ok(sequence) => Ok(sequence),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub async fn get_sleep_cost(&self) -> Option<i32> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSleepCost { respond_to: tx });
        match rx.await {
            Ok(cost) => cost,
            Err(_) => None,
        }
    }

    pub async fn get_state(&self) -> Result<ClientState, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetState { respond_to: tx });
        match rx.await {
            Ok(state) => Ok(state),
            Err(_) => Err("Player disconnected".into()),
        }
    }

    pub fn kick_guild_member(&self, session_id: i32, member_name: String) {
        let _ = self.tx.send(Command::KickGuildMember {
            session_id,
            member_name,
        });
    }

    pub async fn is_trade_accepted(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::IsTradeAccepted { respond_to: tx });
        (rx.await).unwrap_or(false)
    }

    pub async fn is_trading(&self) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::IsTrading { respond_to: tx });
        (rx.await).unwrap_or(false)
    }

    pub fn leave_guild(&self, session_id: i32) {
        let _ = self.tx.send(Command::LeaveGuild { session_id });
    }

    pub fn login(&self, username: String, password: String) {
        let _ = self.tx.send(Command::Login { username, password });
    }

    pub fn pong(&self) {
        let _ = self.tx.send(Command::Pong);
    }

    pub async fn pong_new_sequence(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::PongNewSequence { respond_to: tx });
        let _ = rx.await;
    }

    pub fn request_account_creation(&self, username: String) {
        let _ = self.tx.send(Command::RequestAccountCreation { username });
    }

    pub fn request_character_creation(&self) {
        let _ = self.tx.send(Command::RequestCharacterCreation);
    }

    pub fn request_character_deletion(&self, character_id: i32) {
        let _ = self
            .tx
            .send(Command::RequestCharacterDeletion { character_id });
    }

    pub fn request_guild_creation(&self, session_id: i32, guild_name: String, guild_tag: String) {
        let _ = self.tx.send(Command::RequestGuildCreation {
            session_id,
            guild_name,
            guild_tag,
        });
    }

    pub fn request_guild_details(&self, session_id: i32, guild_identity: String) {
        let _ = self.tx.send(Command::RequestGuildDetails {
            session_id,
            guild_identity,
        });
    }

    pub fn request_guild_memberlist(&self, session_id: i32, guild_identity: String) {
        let _ = self.tx.send(Command::RequestGuildMemberlist {
            session_id,
            guild_identity,
        });
    }

    pub fn request_guild_info(&self, session_id: i32, info_type: GuildInfoType) {
        let _ = self.tx.send(Command::RequestGuildInfo {
            session_id,
            info_type,
        });
    }

    pub fn request_warp(
        &self,
        map_id: i32,
        coords: Coords,
        local: bool,
        animation: Option<WarpEffect>,
    ) {
        let _ = self.tx.send(Command::RequestWarp {
            map_id,
            coords,
            local,
            animation,
        });
    }

    pub fn select_character(&self, character_id: i32) {
        let _ = self.tx.send(Command::SelectCharacter {
            player_handle: self.clone(),
            character_id,
        });
    }

    pub fn send_buf(&self, action: PacketAction, family: PacketFamily, buf: Bytes) {
        let _ = self.tx.send(Command::Send(action, family, buf));
    }

    pub fn send<T>(&self, action: PacketAction, family: PacketFamily, packet: &T)
    where
        T: EoSerialize,
    {
        let mut writer = EoWriter::new();

        if let Err(e) = packet.serialize(&mut writer) {
            error!("Failed to serialize packet: {}", e);
            return;
        }

        self.send_buf(action, family, writer.to_byte_array());
    }

    pub fn set_board_id(&self, board_id: i32) {
        let _ = self.tx.send(Command::SetBoardId(board_id));
    }

    pub fn set_chest_index(&self, index: usize) {
        let _ = self.tx.send(Command::SetChestIndex(index));
    }

    pub fn set_busy(&self, busy: bool) {
        let _ = self.tx.send(Command::SetBusy(busy));
    }

    pub fn set_interact_npc_index(&self, index: i32) {
        let _ = self.tx.send(Command::SetInteractNpcIndex(index));
    }

    pub fn set_interact_player_id(&self, id: Option<i32>) {
        let _ = self.tx.send(Command::SetInteractPlayerId(id));
    }

    pub fn set_party_request(&self, request: PartyRequest) {
        let _ = self.tx.send(Command::SetPartyRequest(request));
    }

    pub fn set_sleep_cost(&self, cost: i32) {
        let _ = self.tx.send(Command::SetSleepCost(cost));
    }

    pub fn set_trade_accepted(&self, accepted: bool) {
        let _ = self.tx.send(Command::SetTradeAccepted(accepted));
    }

    pub fn set_trading(&self, trading: bool) {
        let _ = self.tx.send(Command::SetTrading(trading));
    }

    pub fn tick(&self) {
        let _ = self.tx.send(Command::Tick);
    }

    pub fn update_party_hp(&self, hp_percentage: i32) {
        let _ = self.tx.send(Command::UpdatePartyHP { hp_percentage });
    }

    pub fn update_guild(
        &self,
        session_id: i32,
        info_type_data: GuildAgreeClientPacketInfoTypeData,
    ) {
        let _ = self.tx.send(Command::UpdateGuild {
            session_id,
            info_type_data,
        });
    }

    pub fn assign_guild_rank(&self, session_id: i32, member_name: String, rank: i32) {
        let _ = self.tx.send(Command::AssignGuildRank {
            session_id,
            member_name,
            rank,
        });
    }
}

async fn run_player(mut player: Player, player_handle: PlayerHandle) {
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    trace!("Recv: {:?}", &packet[..4]);
                    player.queue.get_mut().push_back(packet);
                },
                Some(Err(e)) => {
                    match e.kind() {
                        std::io::ErrorKind::BrokenPipe => {
                            player_handle.close("Closed by peer".to_string());
                        },
                        _ => {
                            player_handle.close(format!("Due to unknown error: {:?}", e));
                        }
                    }
                },
                None => {
                }
            },
            Some(command) = player.rx.recv() => {
                // TODO: really don't like how this reads.. maybe a better way to do this?
                if !player.handle_command(command).await {
                    break;
                }
            }
        }

        if player.busy {
            continue;
        }

        if let Some(packet) = player.queue.get_mut().pop_front() {
            player.busy = true;
            let _ = tokio::task::Builder::new()
                .name("handle_packet")
                .spawn(handle_packet(
                    packet,
                    player_handle.clone(),
                    player.world.clone(),
                ));
        }

        tokio::time::sleep(Duration::from_millis(60)).await;
    }
}
