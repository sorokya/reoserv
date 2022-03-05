use std::{cell::RefCell, collections::VecDeque};

use eo::data::{EOChar, EOShort, StreamBuilder};
use tokio::{
    net::TcpStream,
    sync::mpsc::{UnboundedReceiver, UnboundedSender},
};

use crate::{world::WorldHandle, PacketBuf};

use super::{handle_packet::handle_packet, packet_bus::PacketBus, Command, State};

pub struct Player {
    pub id: EOShort,
    pub rx: UnboundedReceiver<Command>,
    pub queue: RefCell<VecDeque<PacketBuf>>,
    pub bus: PacketBus,
    pub tx: UnboundedSender<Command>,
    pub world: WorldHandle,
    pub busy: bool,
    state: State,
    ip: String,
    account_id: EOShort,
    num_of_characters: EOChar,
    character_id: u32,
}

impl Player {
    pub fn new(
        id: EOShort,
        socket: TcpStream,
        rx: UnboundedReceiver<Command>,
        tx: UnboundedSender<Command>,
        world: WorldHandle,
    ) -> Self {
        let ip = socket.peer_addr().unwrap().ip().to_string();
        Self {
            id,
            rx,
            tx,
            world,
            queue: RefCell::new(VecDeque::new()),
            bus: PacketBus::new(socket),
            state: State::Uninitialized,
            ip,
            account_id: 0,
            character_id: 0,
            num_of_characters: 0,
            busy: false,
        }
    }

    pub async fn handle_command(&mut self, command: Command) -> bool {
        match command {
            Command::Send(action, family, data) => {
                let _ = self.bus.send(action, family, data).await;
            }
            Command::PongNewSequence { respond_to } => {
                self.bus.sequencer.pong_new_sequence();
                respond_to.send(()).unwrap();
            }
            Command::GenSequence { respond_to } => {
                let sequence = self.bus.sequencer.gen_sequence();
                respond_to.send(sequence).unwrap();
            }
            Command::Close(reason) => {
                info!("player {} connection closed: {:?}", self.id, reason);
                return false;
            }
            Command::GetEncodeMultiples { respond_to } => {
                respond_to
                    .send([
                        self.bus.packet_processor.encode_multiple,
                        self.bus.packet_processor.decode_multiple,
                    ])
                    .unwrap();
            }
            Command::EnsureValidSequenceForAccountCreation { respond_to } => {
                if self.bus.sequencer.too_big_for_account_reply() {
                    self.bus.sequencer.account_reply_new_sequence();
                }
                let _ = respond_to.send(());
            }
            Command::GetSequenceStart { respond_to } => {
                let _ = respond_to.send(self.bus.sequencer.get_sequence_start());
            }
            Command::GetSequenceBytes { respond_to } => {
                respond_to
                    .send(self.bus.sequencer.get_init_sequence_bytes())
                    .unwrap();
            }
            Command::SetState(state) => {
                self.state = state;
            }
            Command::SetBusy(busy) => {
                self.busy = busy;
            }
            Command::Ping => {
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
            Command::GetIpAddr { respond_to } => {
                let _ = respond_to.send(self.ip.clone());
            }
        }

        true
    }
}
