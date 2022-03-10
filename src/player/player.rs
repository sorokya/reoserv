use std::{cell::RefCell, collections::VecDeque, sync::Arc};

use eo::{
    data::{EOChar, EOInt, EOShort, StreamBuilder},
    net::Weight,
};
use tokio::{
    net::TcpStream,
    sync::{mpsc::UnboundedReceiver, Mutex},
};

use crate::{character::Character, utils, world::WorldHandle, PacketBuf, map::MapHandle};

use super::{packet_bus::PacketBus, Command, InvalidStateError, State};

pub struct Player {
    pub id: EOShort,
    pub rx: UnboundedReceiver<Command>,
    pub queue: RefCell<VecDeque<PacketBuf>>,
    pub bus: PacketBus,
    pub world: WorldHandle,
    pub map: Option<MapHandle>,
    pub busy: bool,
    pub account_id: EOInt,
    pub character_id: EOInt,
    state: State,
    ip: String,
    character: Option<Arc<Mutex<Character>>>,
}

impl Player {
    pub fn new(
        id: EOShort,
        socket: TcpStream,
        rx: UnboundedReceiver<Command>,
        world: WorldHandle,
    ) -> Self {
        let ip = socket.peer_addr().unwrap().ip().to_string();
        Self {
            id,
            rx,
            queue: RefCell::new(VecDeque::new()),
            bus: PacketBus::new(socket),
            world,
            map: None,
            busy: false,
            account_id: 0,
            character_id: 0,
            state: State::Uninitialized,
            ip,
            character: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::CalculateStats { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let mut character = character.lock().await;
                    character.calculate_stats(self.world.clone());
                    let _ = respond_to.send(Ok(()));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
            }
            Command::Close(reason) => {
                info!("player {} connection closed: {:?}", self.id, reason);
                return false;
            }
            Command::EnsureValidSequenceForAccountCreation { respond_to } => {
                if self.bus.sequencer.too_big_for_account_reply() {
                    self.bus.sequencer.account_reply_new_sequence();
                }
                let _ = respond_to.send(());
            }
            Command::GetAccountId { respond_to } => {
                if let State::LoggedIn | State::Playing = self.state {
                    let _ = respond_to.send(Ok(self.account_id));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::LoggedIn, self.state)));
                }
            }
            Command::GetCharacterMapInfo { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(Ok(character.to_map_info(self.id)));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
            }
            Command::GetCoords { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(Ok(character.coords));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
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
            Command::GetItems { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(Ok(character.items.clone()));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
            }
            Command::GetMap { respond_to } => {
                if let Some(map) = self.map.as_ref() {
                    let _ = respond_to.send(Ok(map.to_owned()));
                } else {
                    let _ = respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
            }
            Command::GetMapId { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(Ok(character.map_id));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
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
            Command::GetSequenceStart { respond_to } => {
                let _ = respond_to.send(self.bus.sequencer.get_sequence_start());
            }
            Command::GetSpells { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(Ok(character.spells.clone()));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
            }
            Command::GetWeight { respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(Ok(Weight {
                        current: character.weight as EOChar,
                        max: character.max_weight as EOChar,
                    }));
                } else {
                    let _ =
                        respond_to.send(Err(InvalidStateError::new(State::Playing, self.state)));
                }
            }
            Command::GenSequence { respond_to } => {
                let sequence = self.bus.sequencer.gen_sequence();
                let _ = respond_to.send(sequence);
            }
            Command::IsInRange { coords, respond_to } => {
                if let Some(character) = self.character.as_ref() {
                    let character = character.lock().await;
                    let _ = respond_to.send(utils::in_range(
                        coords.x.into(),
                        coords.y.into(),
                        character.coords.x.into(),
                        character.coords.y.into(),
                    ));
                } else {
                    let _ = respond_to.send(false);
                }
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
                self.character = Some(Arc::new(Mutex::new(character)));
            }
            Command::SetMap(map) => {
                self.map = Some(map);
            }
            Command::SetState(state) => {
                self.state = state;
            }
            Command::Ping => {
                if self.state == State::Uninitialized {
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
                            eo::net::Action::Player,
                            eo::net::Family::Connection,
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
        }

        true
    }
}
