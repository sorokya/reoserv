use anyhow::anyhow;
use bytes::{BufMut, Bytes, BytesMut};
use chrono::Utc;
use eolib::{
    data::{decode_number, encode_number, EoSerialize, EoWriter},
    encrypt::{decrypt_packet, encrypt_packet},
    packet::{generate_sequence_start, Sequencer},
    protocol::net::{PacketAction, PacketFamily},
};
use futures::{SinkExt, StreamExt};
use tokio::net::TcpStream;
use tokio_tungstenite::{tungstenite::Message, WebSocketStream};

use crate::PACKET_RATE_LIMITS;

use super::PacketLog;

pub enum Socket {
    Standard(TcpStream),
    Web(WebSocketStream<TcpStream>),
}

pub struct PacketBus {
    socket: Socket,
    pub log: PacketLog,
    pub need_pong: bool,
    pub sequencer: Sequencer,
    pub upcoming_sequence_start: i32,
    pub server_enryption_multiple: u8,
    pub client_enryption_multiple: u8,
}

impl PacketBus {
    pub fn new(socket: Socket) -> Self {
        let sequencer = Sequencer::new(generate_sequence_start());
        Self {
            socket,
            log: PacketLog::new(),
            need_pong: false,
            sequencer,
            upcoming_sequence_start: 0,
            server_enryption_multiple: 0,
            client_enryption_multiple: 0,
        }
    }

    pub async fn send<T>(
        &mut self,
        action: PacketAction,
        family: PacketFamily,
        packet: T,
    ) -> anyhow::Result<()>
    where
        T: EoSerialize,
    {
        let mut writer = EoWriter::new();

        packet.serialize(&mut writer)?;

        self.send_buf(action, family, writer.to_byte_array()).await
    }

    pub async fn send_buf(
        &mut self,
        action: PacketAction,
        family: PacketFamily,
        data: Bytes,
    ) -> anyhow::Result<()> {
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
        if self.server_enryption_multiple != 0 {
            encrypt_packet(&mut data_buf, self.server_enryption_multiple);
        }
        buf.unsplit(data_buf);

        match &mut self.socket {
            Socket::Standard(socket) => match socket.try_write(&buf) {
                Ok(num_of_bytes_written) => {
                    if num_of_bytes_written != packet_size + 2 {
                        return Err(anyhow!(
                            "Written bytes {} do not match packet size {}",
                            num_of_bytes_written,
                            packet_size
                        ));
                    }
                }
                Err(e) => {
                    return Err(e.into());
                }
            },
            Socket::Web(socket) => {
                if let Err(e) = socket.send(Message::Binary(buf.into())).await {
                    return Err(e.into());
                }
            }
        }

        Ok(())
    }

    pub async fn recv(&mut self) -> Option<std::io::Result<Bytes>> {
        match &mut self.socket {
            Socket::Web(socket) => {
                if let Some(Ok(Message::Binary(buf))) = socket.next().await {
                    let mut data_buf = buf[2..].to_vec();

                    if self.client_enryption_multiple != 0 {
                        decrypt_packet(&mut data_buf, self.client_enryption_multiple);
                    }

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
                } else {
                    None
                }
            }
            Socket::Standard(_) => {
                let get_packet_length = self.get_packet_length();
                let packet_length = get_packet_length.await;
                match packet_length {
                    Some(Ok(packet_length)) => {
                        if packet_length > 0 {
                            let read = self.read(packet_length);
                            match read.await {
                                Some(Ok(buf)) => {
                                    let mut data_buf = buf;
                                    if self.client_enryption_multiple != 0 {
                                        decrypt_packet(
                                            &mut data_buf,
                                            self.client_enryption_multiple,
                                        );
                                    }

                                    let data_buf = Bytes::from(data_buf);

                                    if let Some(rate_limit) =
                                        PACKET_RATE_LIMITS.packets.iter().find(|l| {
                                            l.action == PacketAction::from(data_buf[0])
                                                && l.family == PacketFamily::from(data_buf[1])
                                        })
                                    {
                                        if let Some(last_processed) =
                                            self.log.last_processed(&data_buf)
                                        {
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
        if let Socket::Standard(socket) = &self.socket {
            socket.readable().await.unwrap();
            match socket.try_read(&mut buf) {
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
        } else {
            None
        }
    }
}
