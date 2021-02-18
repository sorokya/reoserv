use std::{io::Read, net::TcpStream};

use eo::{
    data::EOByte,
    net::{ClientState, PACKET_LENGTH_SIZE, PacketProcessor},
};

pub struct Client {
    stream: TcpStream,
    state: ClientState,
    processor: PacketProcessor,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            state: ClientState::Uninitialized,
            processor: PacketProcessor::new(),
        }
    }

    pub fn tick(&mut self) -> std::io::Result<()> {
        self.receive_and_process()?;
        Ok(())
    }

    fn receive_and_process(&mut self) -> std::io::Result<()> {
        let data_length = self.get_packet_length()?;
        if data_length > 0 {
            let mut data_buf = self.receive(data_length)?;
            self.processor.decode(&mut data_buf);

            if data_buf.len() > 0 {
                debug!("Recv: {:?}", data_buf);
            }
        }

        Ok(())
    }

    fn get_packet_length(&mut self) -> std::io::Result<usize> {
        let length_buf = self.receive(PACKET_LENGTH_SIZE)?;
        Ok(eo::data::decode_number(&length_buf) as usize)
    }

    fn receive(&mut self, length: usize) -> std::io::Result<Vec<EOByte>> {
        let mut buf: Vec<EOByte> = vec![0; length];
        match self.stream.read_exact(&mut buf) {
            Ok(_) => {}
            Err(_) => {}
        }
        Ok(buf)
    }
}
