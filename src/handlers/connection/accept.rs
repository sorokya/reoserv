use crate::client::Client;
use client::connection;
use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client,
};

pub struct Accept<'a> {
    client: &'a mut Client,
    packet: client::connection::Accept,
}

impl<'a> Accept<'a> {
    pub fn new(client: &'a mut Client, reader: &'a mut StreamReader<'a>) -> Self {
        let mut packet = client::connection::Accept::new();
        packet.deserialize(reader);
        Self { client, packet }
    }
    pub fn handle_packet(&mut self) -> std::io::Result<()> {
        if self.packet.player_id != self.client.player_id {
            return self.client.close_with_reason(format!(
                "invalid player id. Got {}, expected {}.",
                self.packet.player_id, self.client.player_id
            ));
        }
        let expected_multiples = [
            self.client.processor.encode_multiple,
            self.client.processor.decode_multiple,
        ];
        if self.packet.encoding_multiples != expected_multiples {
            return self.client.close_with_reason(format!(
                "invalid encode multiples. Got {:?}, expected {:?}.",
                self.packet.encoding_multiples, expected_multiples
            ));
        }
        Ok(())
    }
}
