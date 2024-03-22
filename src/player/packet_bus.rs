use bytes::{BufMut, Bytes, BytesMut};
use chrono::Utc;
use eolib::{
    data::{decode_number, encode_number},
    encrypt::{decrypt_packet, encrypt_packet, generate_swap_multiple},
    packet::{generate_sequence_start, Sequencer},
    protocol::net::{PacketAction, PacketFamily},
};
use tokio::net::TcpStream;

use crate::PACKET_RATE_LIMITS;

use super::PacketLog;

pub struct PacketBus {
    socket: TcpStream,
    pub log: PacketLog,
    pub need_pong: bool,
    pub sequencer: Sequencer,
    pub upcoming_sequence_start: i32,
    pub server_enryption_multiple: u8,
    pub client_enryption_multiple: u8,
}

impl PacketBus {
    pub fn new(socket: TcpStream) -> Self {
        let sequencer = Sequencer::new(generate_sequence_start());
        Self {
            socket,
            log: PacketLog::new(),
            need_pong: false,
            sequencer,
            upcoming_sequence_start: 0,
            server_enryption_multiple: generate_swap_multiple(),
            client_enryption_multiple: generate_swap_multiple(),
        }
    }

    pub async fn send(
        &mut self,
        action: PacketAction,
        family: PacketFamily,
        data: Bytes,
    ) -> std::io::Result<()> {
        let packet_size = 2 + data.len();
        let length_bytes = match encode_number(packet_size as i32) {
            Ok(bytes) => bytes,
            Err(e) => {
                error!("Packet send aborted! Error encoding packet size: {}", e);
                return Ok(());
            }
        };

        let mut buf = BytesMut::with_capacity(2 + packet_size);
        buf.put_slice(length_bytes[0..2].as_ref());
        buf.put_u8(u8::from(action));
        buf.put_u8(u8::from(family));
        buf.put(data);

        trace!("Send: {:?}", &buf[..]);

        let mut data_buf = buf.split_off(2);
        encrypt_packet(&mut data_buf, self.server_enryption_multiple);
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
        let get_packet_length = self.get_packet_length();
        let packet_length = get_packet_length.await;
        match packet_length {
            Some(Ok(packet_length)) => {
                if packet_length > 0 {
                    let read = self.read(packet_length);
                    match read.await {
                        Some(Ok(buf)) => {
                            let mut data_buf = buf;
                            decrypt_packet(&mut data_buf, self.client_enryption_multiple);

                            let data_buf = Bytes::from(data_buf);

                            if let Some(rate_limit) = PACKET_RATE_LIMITS.packets.iter().find(|l| {
                                l.action == PacketAction::from(data_buf[0])
                                    && l.family == PacketFamily::from(data_buf[1])
                            }) {
                                if let Some(last_processed) = self.log.last_processed(&data_buf) {
                                    if Utc::now()
                                        .signed_duration_since(last_processed)
                                        .num_milliseconds()
                                        < rate_limit.limit
                                    {
                                        let mut buf = BytesMut::new();
                                        buf.put_u8(0xfe);
                                        buf.put_u8(0xfe);
                                        buf.put_u8(data_buf[2]);

                                        return Some(Ok(buf.freeze()));
                                    }
                                }

                                self.log.add_entry(&data_buf);
                            }

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
            Some(Ok(buf)) => Some(Ok(decode_number(&buf) as usize)),
            Some(Err(e)) => Some(Err(e)),
            None => None,
        }
    }

    async fn read(&self, length: usize) -> Option<std::io::Result<Vec<u8>>> {
        let mut buf: Vec<u8> = vec![0; length];
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
