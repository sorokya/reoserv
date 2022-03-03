use eo::{
    data::{encode_number, EOByte, StreamBuilder},
    net::{
        packets::server::Sequencer, Action, Family, PacketProcessor, PACKET_HEADER_SIZE,
        PACKET_LENGTH_SIZE,
    },
};
use tokio::net::TcpStream;

use crate::PacketBuf;

pub struct PacketBus {
    socket: TcpStream,
    pub need_pong: bool,
    pub sequencer: Sequencer,
    pub packet_processor: PacketProcessor,
}

impl PacketBus {
    pub fn new(socket: TcpStream) -> Self {
        let mut sequencer = Sequencer::default();
        sequencer.init_new_sequence();
        Self {
            socket,
            need_pong: false,
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
        trace!("Send: {:?}", buf);
        self.packet_processor.encode(&mut buf);

        let length_bytes = encode_number(packet_size as u32);
        buf.insert(0, length_bytes[1]);
        buf.insert(0, length_bytes[0]);

        match self.socket.try_write(&buf) {
            Ok(num_of_bytes_written) => {
                if num_of_bytes_written != packet_size + PACKET_LENGTH_SIZE {
                    error!(
                        "Written bytes ({}) doesn't match packet size ({})",
                        num_of_bytes_written, packet_size
                    );
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
            Some(Ok(packet_length)) => {
                if packet_length > 0 {
                    match self.read(packet_length).await {
                        Some(Ok(buf)) => {
                            let mut data_buf = buf;
                            self.packet_processor.decode(&mut data_buf);
                            Some(Ok(data_buf))
                        }
                        Some(Err(e)) => Some(Err(e)),
                        None => None,
                    }
                } else {
                    None
                }
            }
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    async fn get_packet_length(&self) -> Option<std::io::Result<usize>> {
        match self.read(2).await {
            Some(Ok(buf)) => Some(Ok(eo::data::decode_number(&buf) as usize)),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    async fn read(&self, length: usize) -> Option<std::io::Result<Vec<EOByte>>> {
        let mut buf: Vec<EOByte> = vec![0; length];
        self.socket.readable().await.unwrap();
        match self.socket.try_read(&mut buf) {
            Ok(0) => {
                return Some(Err(std::io::Error::new(
                    std::io::ErrorKind::BrokenPipe,
                    "Connection closed",
                )));
            }
            Ok(_) => {}
            Err(_) => {
                return None;
            }
        }
        Some(Ok(buf))
    }
}
