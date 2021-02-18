use std::{io::Read, net::TcpStream};

use eo::{
    data::EOByte,
    net::{ClientState, PacketProcessor},
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
        let length_buf = self.receive(2)?;
        let data_length = eo::data::decode_number(&length_buf) as usize;
        let mut data_buf = self.receive(data_length)?;
        self.processor.decode(&mut data_buf);

        if data_buf.len() > 0 {
            debug!("Recv: {:?}", data_buf);
        }

        Ok(())
    }

    fn receive(&mut self, length: usize) -> std::io::Result<Vec<EOByte>> {
        let mut buf: Vec<EOByte> = vec![0; length];
        self.stream.read_exact(&mut buf)?;
        Ok(buf)
    }
}
