use eo::{data::{EOByte, EOShort}, net::{PacketProcessor, packets::server::Sequencer}};
use tokio::{net::TcpStream, sync::mpsc};

use crate::{Players, Rx, PacketBuf};

pub struct Player {
    player_id: EOShort,
    pub rx: Rx,
    pub bus: PacketBus,
}

impl Player {
    pub fn new(players: Players, socket: TcpStream, player_id: EOShort) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        players.lock().expect("Failed to lock players").insert(player_id, tx);

        Self {
            player_id,
            rx,
            bus: PacketBus::new(socket),
        }
    }
}

pub struct PacketBus {
    socket: TcpStream,
    sequencer: Sequencer,
    packet_processor: PacketProcessor,
}

impl PacketBus {
    pub fn new(socket: TcpStream) -> Self {
        Self {
            socket,
            sequencer: Sequencer::default(),
            packet_processor: PacketProcessor::new(),
        }
    }

    pub async fn send(&mut self, packet: PacketBuf) -> std::io::Result<()> {
        Ok(())
    }

    pub async fn recv(&self) -> Option<std::io::Result<PacketBuf>> {
        match self.get_packet_length().await {
            Some(packet_length) => {
                let data_buf = self.read(packet_length).await.unwrap();
                Some(Ok(data_buf))
            },
            None => {
                return None;
            }
        }
    }

    async fn get_packet_length(&self) -> Option<usize> {
        if let Some(length_buf) = self.read(2).await {
            Some(eo::data::decode_number(&length_buf) as usize)
        } else {
            None
        }
    }

    async fn read(&self, length: usize) -> Option<Vec<EOByte>> {
        self.socket.readable().await.unwrap();
        let mut buf: Vec<EOByte> = vec![0; length];
        match self.socket.try_read(&mut buf) {
            Ok(_) => {}
            Err(_) => {
                return None;
            }
        }
        Some(buf)
    }
}