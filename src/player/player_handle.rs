use bytes::Bytes;
use eolib::protocol::{
    net::{server::WarpEffect, PacketAction, PacketFamily, Version},
    Coords,
};
use mysql_async::Pool;
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::{
    character::Character,
    errors::{InvalidStateError, MissingSessionIdError},
    map::MapHandle,
    world::WorldHandle,
};

use super::{handle_packet::handle_packet, player::Player, ClientState, Command, PartyRequest};

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(id: i32, socket: TcpStream, world: WorldHandle, pool: Pool) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let player = Player::new(id, socket, rx, world, pool);
        let _ = tokio::task::Builder::new()
            .name(&format!("Player {}", id))
            .spawn(run_player(player, PlayerHandle::for_tx(tx.clone())));

        Self { tx }
    }

    fn for_tx(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    pub fn accept_warp(&self, map_id: i32, session_id: i32) {
        let _ = self.tx.send(Command::AcceptWarp { map_id, session_id });
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

    pub fn arena_die(&self, spawn_coords: Coords) {
        let _ = self.tx.send(Command::ArenaDie { spawn_coords });
    }

    pub fn die(&self) {
        let _ = self.tx.send(Command::Die);
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

    pub async fn get_account_id(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetAccountId { respond_to: tx });
        match rx.await {
            Ok(Ok(account_id)) => Ok(account_id),
            Ok(Err(e)) => Err(Box::new(e)),
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

    pub async fn get_ip_addr(&self) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetIpAddr { respond_to: tx });
        match rx.await {
            Ok(ip_addr) => Ok(ip_addr),
            Err(_) => Err("Player disconnected".into()),
        }
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

    pub async fn get_map_id(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMapId { respond_to: tx });
        match rx.await {
            Ok(Ok(map_id)) => Ok(map_id),
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

    pub async fn get_sequence_start(
        &self,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSequenceStart { respond_to: tx });
        match rx.await {
            Ok(sequence) => Ok(sequence),
            Err(_) => Err("Player disconnected".into()),
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

    pub fn ping(&self) {
        let _ = self.tx.send(Command::Ping);
    }

    pub fn pong(&self) {
        let _ = self.tx.send(Command::Pong);
    }

    pub async fn pong_new_sequence(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::PongNewSequence { respond_to: tx });
        let _ = rx.await;
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

    pub fn send(&self, action: PacketAction, family: PacketFamily, buf: Bytes) {
        let _ = self.tx.send(Command::Send(action, family, buf));
    }

    pub fn set_account_id(&self, account_id: i32) {
        let _ = self.tx.send(Command::SetAccountId(account_id));
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

    pub fn set_character(&self, character: Box<Character>) {
        let _ = self.tx.send(Command::SetCharacter(character));
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

    pub fn set_map(&self, map: MapHandle) {
        let _ = self.tx.send(Command::SetMap(map));
    }

    pub fn set_state(&self, state: ClientState) {
        let _ = self.tx.send(Command::SetState(state));
    }

    pub async fn take_character(&self) -> Result<Box<Character>, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::TakeCharacter { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn take_session_id(&self) -> Result<i32, MissingSessionIdError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::TakeSessionId { respond_to: tx });
        rx.await.unwrap()
    }

    pub fn update_party_hp(&self, hp_percentage: i32) {
        let _ = self.tx.send(Command::UpdatePartyHP { hp_percentage });
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
    }
}
