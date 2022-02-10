use eo::{
    data::{encode_number, EOByte, EOInt, EOShort, StreamBuilder, StreamReader, MAX1},
    net::{
        packets::server::Sequencer, Action, Family, PacketProcessor, PACKET_HEADER_SIZE,
        PACKET_LENGTH_SIZE,
    },
};
use num_traits::FromPrimitive;
use tokio::{io::AsyncWriteExt, net::TcpStream, sync::mpsc};

use crate::{PacketBuf, Players, Rx};

#[derive(Debug)]
pub enum Command {
    InitNewSequence,
    Send(Action, Family, PacketBuf),
}

pub struct Player {
    pub rx: Rx,
    pub bus: PacketBus,
}

impl Player {
    pub async fn new(players: Players, socket: TcpStream, player_id: EOShort) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        players.lock().await.insert(player_id, tx);

        Self {
            rx,
            bus: PacketBus::new(socket),
        }
    }
}

pub struct PacketBus {
    socket: TcpStream,
    pub sequencer: Sequencer,
    pub packet_processor: PacketProcessor,
}

impl PacketBus {
    pub fn new(socket: TcpStream) -> Self {
        let mut sequencer = Sequencer::default();
        sequencer.init_new_sequence();
        Self {
            socket,
            sequencer,
            packet_processor: PacketProcessor::new(),
        }
    }

    pub async fn send(
        &mut self,
        action: Action,
        family: Family,
        mut data: PacketBuf,
    ) -> std::io::Result<()> {
        let packet_size = PACKET_HEADER_SIZE + data.len();
        let mut builder = StreamBuilder::with_capacity(PACKET_LENGTH_SIZE + packet_size);

        builder.add_byte(action as EOByte);
        builder.add_byte(family as EOByte);
        builder.append(&mut data);

        let mut buf = builder.get();
        debug!("Send: {:?}", buf);
        self.packet_processor.encode(&mut buf);

        let length_bytes = encode_number(packet_size as u32);
        buf.insert(0, length_bytes[1]);
        buf.insert(0, length_bytes[0]);

        self.socket.writable().await.unwrap();
        match self.socket.try_write(&buf) {
            Ok(num_of_bytes_written) => {
                if num_of_bytes_written != packet_size + PACKET_LENGTH_SIZE {
                    error!("Written bytes ({}) doesn't match packet size ({})", num_of_bytes_written, packet_size);
                }
            }
            _ => {
                error!("Error writing to socket");
            }
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Option<std::io::Result<PacketBuf>> {
        match self.get_packet_length().await {
            Some(packet_length) => {
                if packet_length > 0 {
                    let mut data_buf = self.read(packet_length).await.unwrap();
                    self.packet_processor.decode(&mut data_buf);

                    let action = Action::from_u8(data_buf[0]).unwrap();
                    let family = Family::from_u8(data_buf[1]).unwrap();
                    let reader = StreamReader::new(&data_buf[2..]);

                    if family != Family::Init {
                        if family == Family::Connection && action == Action::Ping {
                            self.sequencer.pong_new_sequence();
                        }

                        let server_sequence = self.sequencer.gen_sequence();
                        let client_sequence = if server_sequence > MAX1 {
                            reader.get_short() as EOInt
                        } else {
                            reader.get_char() as EOInt
                        };

                        if server_sequence != client_sequence {
                            // TODO
                            // return self.close_with_reason(format!(
                            //     "sending invalid sequence: Got {}, expected {}.",
                            //     client_sequence, server_sequence
                            // ));
                        }
                    } else {
                        self.sequencer.gen_sequence();
                    }

                    Some(Ok(data_buf))
                } else {
                    None
                }
            }
            None => None,
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
