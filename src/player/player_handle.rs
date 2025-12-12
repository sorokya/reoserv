use bytes::Bytes;
use chrono::{DateTime, Utc};
use eolib::{
    data::{EoSerialize, EoWriter},
    protocol::{
        net::{
            server::{GuildReply, WarpEffect},
            PacketAction, PacketFamily,
        },
        Coords,
    },
};
use eoplus::Arg;
use mysql_async::Pool;
use std::time::Duration;
use tokio::sync::{mpsc, oneshot};
use tokio::time::timeout;

use crate::{character::Character, map::MapHandle, world::WorldHandle};

use super::{player::Player, ClientState, Command, PartyRequest, Socket};

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(
        id: i32,
        socket: Socket,
        ip: String,
        connected_at: DateTime<Utc>,
        world: WorldHandle,
        pool: Pool,
    ) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let player = Player::new(id, socket, ip, connected_at, rx, world, pool);
        tokio::spawn(run_player(player));

        Self { tx }
    }

    pub fn add_guild_creation_player(&self, player_id: i32, name: String) {
        let _ = self
            .tx
            .send(Command::AddGuildCreationPlayer { player_id, name });
    }

    pub fn arena_die(&self, spawn_coords: Coords) {
        let _ = self.tx.send(Command::ArenaDie { spawn_coords });
    }

    pub fn cancel_trade(&self) {
        let _ = self.tx.send(Command::CancelTrade);
    }

    pub fn close(&self, reason: String) {
        let _ = self.tx.send(Command::Close(reason));
    }

    pub fn die(&self) {
        let _ = self.tx.send(Command::Die);
    }

    pub async fn generate_session_id(
        &self,
    ) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GenerateSessionId { respond_to: tx });
        timeout(Duration::from_secs(5), rx)
            .await
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Failed to generate session id. Timeout".into() })?
            .map_err(|_| -> Box<dyn std::error::Error + Send + Sync> { "Failed to generate session id. Channel closed".into() })
    }

    pub async fn get_character(
        &self,
    ) -> Result<Box<Character>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacter { respond_to: tx });
        match timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(Ok(character))) => Ok(character),
            Ok(Ok(Err(e))) => Err(Box::new(e)),
            Ok(Err(_)) => Err("Failed to get character. Channel closed".into()),
            Err(_) => Err("Failed to get character. Timeout".into()),
        }
    }

    pub async fn get_map(&self) -> Result<MapHandle, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMap { respond_to: tx });
        match timeout(Duration::from_secs(10), rx).await {
            Ok(Ok(Ok(map))) => Ok(map),
            Ok(Ok(Err(e))) => Err(Box::new(e)),
            Ok(Err(_)) => Err("Failed to get map. Channel closed".into()),
            Err(_) => Err("Failed to get map. Timeout".into()),
        }
    }

    pub async fn get_party_request(&self) -> Result<PartyRequest, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPartyRequest { respond_to: tx });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(party_request)) => Ok(party_request),
            Ok(Err(_)) => Err("Failed to get party request. Channel closed".into()),
            Err(_) => Err("Failed to get party request. Timeout".into()),
        }
    }

    pub async fn get_player_id(&self) -> Result<i32, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerId { respond_to: tx });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(player_id)) => Ok(player_id),
            Ok(Err(_)) => Err("Failed to get player id. Channel closed".into()),
            Err(_) => Err("Failed to get player id. Timeout".into()),
        }
    }

    pub async fn get_interact_player_id(&self) -> Result<Option<i32>, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetInteractPlayerId { respond_to: tx });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(id)) => Ok(id),
            Ok(Err(_)) => Err("Failed to get interact player id. Channel closed".into()),
            Err(_) => Err("Failed to get interact player id. Timeout".into()),
        }
    }

    pub async fn get_state(&self) -> Result<ClientState, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetState { respond_to: tx });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(state)) => Ok(state),
            Ok(Err(_)) => Err("Failed to get state. Channel closed".into()),
            Err(_) => Err("Failed to get state. Timeout".into()),
        }
    }

    pub async fn is_trade_accepted(&self) -> Result<bool, Box<dyn std::error::Error + Send + Sync>> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::IsTradeAccepted { respond_to: tx });
        match timeout(Duration::from_secs(5), rx).await {
            Ok(Ok(accepted)) => Ok(accepted),
            Ok(Err(_)) => Err("Failed to check if trade accepted. Channel closed".into()),
            Err(_) => Err("Failed to check if trade accepted. Timeout".into()),
        }
    }

    pub fn quest_action(&self, action: String, args: Vec<Arg>) {
        let _ = self.tx.send(Command::QuestAction { action, args });
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

    pub fn send_guild_reply(&self, guild_reply: GuildReply) {
        let _ = self.tx.send(Command::SendGuildReply(guild_reply));
    }

    pub fn send_server_message(&self, message: &str) {
        let _ = self.tx.send(Command::SendServerMessage(message.to_owned()));
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

    pub fn show_captcha(&self, experience: i32) {
        let _ = self.tx.send(Command::ShowCaptcha { experience });
    }

    pub fn tick(&self) {
        let _ = self.tx.send(Command::Tick);
    }

    pub fn update_chest_content(&self, chest_index: usize, buf: Bytes) {
        let _ = self
            .tx
            .send(Command::UpdateChestContent { chest_index, buf });
    }

    pub fn update_party_hp(&self, hp_percentage: i32) {
        let _ = self.tx.send(Command::UpdatePartyHP { hp_percentage });
    }
}

async fn run_player(mut player: Player) {
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    trace!("Recv: {:?}", &packet[4..]);
                    player.queue.get_mut().push_back(packet);
                },
                Some(Err(e)) => {
                    match e.kind() {
                        std::io::ErrorKind::BrokenPipe => {
                            player.close("Closed by peer".to_string()).await;
                            break;
                        },
                        _ => {
                            player.close(format!("Due to unknown error: {:?}", e)).await;
                            break;
                        }
                    }
                },
                None => {
                }
            },
            Some(command) = player.rx.recv() => {
                player.handle_command(command).await;
            }
        }

        if player.closed {
            break;
        }

        if let Some(packet) = player.queue.get_mut().pop_front() {
            player.handle_packet(packet).await;
        }
    }
}
