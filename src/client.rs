use eo::{
    data::{encode_number, EOByte, StreamBuilder, StreamReader},
    net::{Action, ClientState, Family, PacketProcessor, PACKET_HEADER_SIZE, PACKET_LENGTH_SIZE},
};
use num_traits::FromPrimitive;
use std::{
    io::{Read, Write},
    net::{Shutdown, TcpStream},
};

use crate::handlers;

pub struct Client {
    stream: TcpStream,
    pub state: ClientState,
    pub processor: PacketProcessor,
    pub closed: bool,
}

impl Client {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            state: ClientState::Uninitialized,
            processor: PacketProcessor::new(),
            closed: false,
        }
    }

    pub fn tick(&mut self) -> std::io::Result<()> {
        self.receive_and_process()?;
        Ok(())
    }

    pub fn send(
        &mut self,
        family: Family,
        action: Action,
        mut data: Vec<EOByte>,
    ) -> std::io::Result<()> {
        let packet_size = PACKET_HEADER_SIZE + data.len();
        let mut builder = StreamBuilder::with_capacity(PACKET_LENGTH_SIZE + packet_size);

        builder.add_byte(action as EOByte);
        builder.add_byte(family as EOByte);
        builder.append(&mut data);

        let mut buf = builder.get();
        debug!("Send: {:?}", buf);
        self.processor.encode(&mut buf);

        let length_bytes = encode_number(packet_size as u32);
        buf.insert(0, length_bytes[1]);
        buf.insert(0, length_bytes[0]);
        self.stream.write_all(&buf)?;

        Ok(())
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        self.closed = true;
        Ok(())
    }

    fn receive_and_process(&mut self) -> std::io::Result<()> {
        let data_length = self.get_packet_length()?;
        if data_length > 0 {
            let mut data_buf = self.receive(data_length)?;
            self.processor.decode(&mut data_buf);
            debug!("Recv: {:?}", data_buf);

            let action = Action::from_u8(data_buf[0]).unwrap();
            let family = Family::from_u8(data_buf[1]).unwrap();
            let mut reader = StreamReader::new(&data_buf[2..]);
            match family {
                Family::Init => match action {
                    Action::Init => handlers::init::Init::new(self, &mut reader).handle_packet()?,
                    _ => error!("No handler for packet: {:?}_{:?}", family, action),
                },
                _ => error!("Unknown family: {:?}", family),
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
