use eo::{data::EOShort, net::{PacketProcessor, packets::server::Sequencer}};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{Players, Rx};

pub struct Player {
    socket: TcpStream,
    player_id: EOShort,
    sequencer: Sequencer,
    packet_processor: PacketProcessor,
    rx: Rx,
}

impl Player {
    pub fn new(players: Players, socket: TcpStream, player_id: EOShort) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        players.lock().expect("Failed to lock players").insert(player_id, tx);

        Self {
            socket,
            player_id,
            sequencer: Sequencer::default(),
            packet_processor: PacketProcessor::new(),
            rx,
        }
    }

    pub async fn tick(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        Ok(())
    }
}