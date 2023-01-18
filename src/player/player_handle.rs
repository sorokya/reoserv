use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    protocol::{Coords, PacketAction, PacketFamily, WarpAnimation},
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
    PacketBuf,
};

use super::{handle_packet::handle_packet, player::Player, Command};

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(id: EOShort, socket: TcpStream, world: WorldHandle, pool: Pool) -> Self {
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

    pub fn accept_warp(&self, map_id: EOShort, session_id: EOShort) {
        let _ = self.tx.send(Command::AcceptWarp { map_id, session_id });
    }

    pub fn close(&self, reason: String) {
        let _ = self.tx.send(Command::Close(reason));
    }

    pub async fn ensure_valid_sequence_for_account_creation(&self) {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::EnsureValidSequenceForAccountCreation { respond_to: tx });
        rx.await.unwrap();
    }

    pub async fn generate_session_id(&self) -> EOShort {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GenerateSessionId { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_account_id(&self) -> Result<EOInt, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetAccountId { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_character(&self) -> Result<Box<Character>, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCharacter { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn gen_encoding_multiples(&self) -> [EOByte; 2] {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GenEncodingMultiples { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_encoding_multiples(&self) -> [EOByte; 2] {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetEncodingMultiples { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_ip_addr(&self) -> String {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetIpAddr { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_map(&self) -> Result<MapHandle, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMap { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_map_id(&self) -> Result<EOShort, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetMapId { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_player_id(&self) -> EOShort {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetPlayerId { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_session_id(&self) -> Result<EOShort, MissingSessionIdError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSessionId { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_sequence_bytes(&self) -> (EOShort, EOChar) {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSequenceBytes { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_sequence_start(&self) -> EOInt {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSequenceStart { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn gen_sequence(&self) -> EOInt {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GenSequence { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_state(&self) -> ClientState {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetState { respond_to: tx });
        rx.await.unwrap()
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
        rx.await.unwrap();
    }

    pub fn request_warp(
        &self,
        map_id: EOShort,
        coords: Coords,
        local: bool,
        animation: Option<WarpAnimation>,
    ) {
        let _ = self.tx.send(Command::RequestWarp {
            map_id,
            coords,
            local,
            animation,
        });
    }

    pub fn send(&self, action: PacketAction, family: PacketFamily, buf: PacketBuf) {
        let _ = self.tx.send(Command::Send(action, family, buf));
    }

    pub fn set_account_id(&self, account_id: EOInt) {
        let _ = self.tx.send(Command::SetAccountId(account_id));
    }

    pub fn set_busy(&self, busy: bool) {
        let _ = self.tx.send(Command::SetBusy(busy));
    }

    pub fn set_character(&self, character: Box<Character>) {
        let _ = self.tx.send(Command::SetCharacter(character));
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

    pub async fn take_session_id(&self) -> Result<EOShort, MissingSessionIdError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::TakeSessionId { respond_to: tx });
        rx.await.unwrap()
    }
}

async fn run_player(mut player: Player, player_handle: PlayerHandle) {
    loop {
        tokio::select! {
            result = player.bus.recv() => match result {
                Some(Ok(packet)) => {
                    trace!("Recv: {:?}", packet);
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
