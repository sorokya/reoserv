use diesel::MysqlConnection;
use eo::{
    data::{encode_number, EOByte, EOChar, EOInt, EOShort, StreamBuilder, StreamReader, MAX1},
    net::{Action, ClientState, Family, PacketProcessor, PACKET_HEADER_SIZE, PACKET_LENGTH_SIZE},
};
use num_traits::FromPrimitive;
use rand::prelude::*;
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
    pub player_id: EOShort,
    pub sequence_start: i32,
    pub upcoming_sequence_start: i32,
    pub sequence: EOInt,
    pub ip_address: String,
}

impl Client {
    pub fn new(stream: TcpStream, player_id: EOShort) -> Self {
        let ip_address = stream
            .peer_addr()
            .expect("Failed to get client IP address")
            .ip()
            .to_string();
        Self {
            stream,
            state: ClientState::Uninitialized,
            processor: PacketProcessor::new(),
            closed: false,
            player_id,
            sequence_start: 0,
            upcoming_sequence_start: 0,
            sequence: 0,
            ip_address,
        }
    }

    pub fn tick(&mut self, db: &MysqlConnection) -> std::io::Result<()> {
        self.receive_and_process(db)?;
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

    pub fn init_new_sequence(&mut self) {
        let mut rng = rand::thread_rng();
        self.sequence_start = rng.gen_range(0, 1757);
    }

    pub fn account_reply_new_sequence(&mut self) {
        let mut rng = rand::thread_rng();
        self.sequence_start = rng.gen_range(0, 240);
    }

    pub fn ping_new_sequence(&mut self) {
        let mut rng = rand::thread_rng();
        self.upcoming_sequence_start = rng.gen_range(0, 1757);
    }

    pub fn pong_new_sequence(&mut self) {
        self.sequence_start = self.upcoming_sequence_start;
    }

    pub fn get_init_sequence_bytes(&self) -> (EOShort, EOChar) {
        let mut rng = rand::thread_rng();
        let s1_max = (self.sequence_start + 13) / 7;
        let s1_min = std::cmp::max(0, (self.sequence_start - 252 + 13 + 6) / 7);
        let s1 = rng.gen_range(s1_min, s1_max);
        let s2 = self.sequence_start - s1 * 7 + 13;
        (s1 as EOShort, s2 as EOChar)
    }

    pub fn get_update_sequence_bytes(&self) -> (EOShort, EOChar) {
        let mut rng = rand::thread_rng();
        let s1_max = self.upcoming_sequence_start + 252;
        let s1_min = self.upcoming_sequence_start;
        let s1 = rng.gen_range(s1_min, s1_max);
        let s2 = s1 - self.upcoming_sequence_start;
        (s1 as EOShort, s2 as EOChar)
    }

    pub fn gen_sequence(&mut self) -> EOInt {
        let result = self.sequence_start as EOInt + self.sequence;
        self.sequence = (self.sequence + 1) % 10;
        result
    }

    pub fn gen_upcoming_sequence(&mut self) -> EOInt {
        let result = self.upcoming_sequence_start as EOInt + self.sequence;
        self.sequence = (self.sequence + 1) % 10;
        result
    }

    pub fn close(&mut self) -> std::io::Result<()> {
        self.stream.shutdown(Shutdown::Both)?;
        self.closed = true;
        Ok(())
    }

    pub fn close_with_reason(&mut self, reason: String) -> std::io::Result<()> {
        warn!(
            "closing client ({}). Reason: {}",
            self.stream.peer_addr()?,
            reason
        );
        self.close()?;
        Ok(())
    }

    fn receive_and_process(&mut self, db: &MysqlConnection) -> std::io::Result<()> {
        let data_length = self.get_packet_length()?;
        if data_length > 0 {
            let mut data_buf = self.receive(data_length)?;
            self.processor.decode(&mut data_buf);
            debug!("Recv: {:?}", data_buf);

            let action = Action::from_u8(data_buf[0]).unwrap();
            let family = Family::from_u8(data_buf[1]).unwrap();
            let mut reader = StreamReader::new(&data_buf[2..]);

            if family != Family::Init {
                if family == Family::Connection && action == Action::Ping {
                    self.pong_new_sequence();
                }

                let server_sequence = self.gen_sequence();
                let client_sequence = if server_sequence > MAX1 {
                    reader.get_short() as EOInt
                } else {
                    reader.get_char() as EOInt
                };

                if server_sequence != client_sequence {
                    return self.close_with_reason(format!(
                        "sending invalid sequence: Got {}, expected {}.",
                        client_sequence, server_sequence
                    ));
                }
            } else {
                self.gen_sequence();
            }

            match family {
                Family::Init => match action {
                    Action::Init => handlers::init::Init::new(self, &mut reader).handle_packet()?,
                    _ => error!("No handler for packet: {:?}_{:?}", family, action),
                },
                Family::Connection => match action {
                    Action::Accept => {
                        handlers::connection::Accept::new(self, &mut reader).handle_packet()?
                    }
                    _ => error!("No handler for packet: {:?}_{:?}", family, action),
                },
                Family::Account => match action {
                    Action::Request => {
                        handlers::account::Request::new(self, &mut reader, db).handle_packet()?
                    }
                    Action::Create => {
                        handlers::account::Create::new(self, &mut reader, db).handle_packet()?
                    }
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
