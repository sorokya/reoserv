use eo::{
    data::{EOByte, EOChar, EOInt, EOShort},
    net::{Action, CharacterMapInfo, Family, Item, Spell, Weight},
    world::Coords,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc, oneshot},
};

use crate::{character::Character, world::WorldHandle, PacketBuf};

use super::{handle_packet::handle_packet, player::Player, Command, InvalidStateError, State};

#[derive(Debug, Clone)]
pub struct PlayerHandle {
    tx: mpsc::UnboundedSender<Command>,
}

impl PlayerHandle {
    pub fn new(id: EOShort, socket: TcpStream, world: WorldHandle) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let player = Player::new(id, socket, rx, world);
        tokio::task::Builder::new()
            .name(&format!("Player {}", id))
            .spawn(run_player(player, PlayerHandle::for_tx(tx.clone())));

        Self { tx }
    }

    fn for_tx(tx: mpsc::UnboundedSender<Command>) -> Self {
        Self { tx }
    }

    pub async fn calculate_stats(&self) -> Result<(), InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::CalculateStats { respond_to: tx });
        rx.await.unwrap()
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

    pub async fn get_account_id(&self) -> Result<EOInt, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetAccountId { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_character_map_info(&self) -> Result<CharacterMapInfo, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self
            .tx
            .send(Command::GetCharacterMapInfo { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_coords(&self) -> Result<Coords, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetCoords { respond_to: tx });
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

    pub async fn get_items(&self) -> Result<Vec<Item>, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetItems { respond_to: tx });
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

    pub async fn get_spells(&self) -> Result<Vec<Spell>, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetSpells { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn get_weight(&self) -> Result<Weight, InvalidStateError> {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GetWeight { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn gen_sequence(&self) -> EOInt {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::GenSequence { respond_to: tx });
        rx.await.unwrap()
    }

    pub async fn is_in_range(&self, coords: Coords) -> bool {
        let (tx, rx) = oneshot::channel();
        let _ = self.tx.send(Command::IsInRange {
            coords,
            respond_to: tx,
        });
        rx.await.unwrap()
    }

    pub fn send(&self, action: Action, family: Family, buf: PacketBuf) {
        let _ = self.tx.send(Command::Send(action, family, buf));
    }

    pub fn set_account_id(&self, account_id: EOInt) {
        let _ = self.tx.send(Command::SetAccountId(account_id));
    }

    pub fn set_busy(&self, busy: bool) {
        let _ = self.tx.send(Command::SetBusy(busy));
    }

    pub fn set_character(&self, character: Character) {
        let _ = self.tx.send(Command::SetCharacter(character));
    }

    pub fn set_state(&self, state: State) {
        let _ = self.tx.send(Command::SetState(state));
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
                            info!("player {} connection closed by peer", player.id);
                            break;
                        },
                        _ => {
                            info!("player {} connection closed due to unknown error: {:?}", player.id, e);
                            break;
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
            tokio::task::Builder::new()
                .name("handle_packet")
                .spawn(handle_packet(
                    packet,
                    player.id,
                    player_handle.clone(),
                    player.world.clone(),
                ));
        }
    }

    player
        .world
        .drop_player(player.id, player.account_id, player.character_id)
        .await
        .unwrap();
}
