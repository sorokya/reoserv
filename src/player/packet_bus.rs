use bytes::{BufMut, Bytes, BytesMut};
use eo::{
    data::{encode_number, EOByte},
    net::{PacketProcessor, ServerSequencer},
    protocol::{PacketAction, PacketFamily},
};
use tokio::net::TcpStream;

pub struct PacketBus {
    socket: TcpStream,
    pub need_pong: bool,
    pub sequencer: ServerSequencer,
    pub packet_processor: PacketProcessor,
}

impl PacketBus {
    pub fn new(socket: TcpStream) -> Self {
        let mut sequencer = ServerSequencer::default();
        sequencer.init_new_sequence();
        Self {
            socket,
            need_pong: false,
            sequencer,
            packet_processor: PacketProcessor::default(),
        }
    }

    pub async fn send(
        &mut self,
        action: PacketAction,
        family: PacketFamily,
        data: Bytes,
    ) -> std::io::Result<()> {
        let packet_size = 2 + data.len();
        let length_bytes = encode_number(packet_size as u32);

        let mut buf = BytesMut::with_capacity(2 + packet_size);
        buf.put_slice(length_bytes[0..2].as_ref());
        buf.put_u8(action.to_byte());
        buf.put_u8(family.to_byte());
        buf.put(data);

        trace!("Send: {:?}", &buf[..]);

        let mut data_buf = buf.split_off(2);
        self.packet_processor.encode(&mut data_buf);
        buf.unsplit(data_buf);

        match self.socket.try_write(&buf) {
            Ok(num_of_bytes_written) => {
                if num_of_bytes_written != packet_size + 2 {
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

    pub async fn recv(&mut self) -> Option<std::io::Result<Bytes>> {
        match self.get_packet_length().await {
            Some(Ok(packet_length)) => {
                if packet_length > 0 {
                    match self.read(packet_length).await {
                        Some(Ok(buf)) => {
                            let mut data_buf = buf;
                            self.packet_processor.decode(&mut data_buf);

                            let data_buf = Bytes::from(data_buf);
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
