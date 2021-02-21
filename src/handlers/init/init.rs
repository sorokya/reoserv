use crate::client::Client;
use eo::{
    data::{EOByte, Serializeable, StreamReader},
    net::{
        packets::{client, server},
        stupid_hash, Action, ClientState, Family, InitReply,
    },
};

pub struct Init<'a> {
    client: &'a mut Client,
    packet: client::InitInit,
}

impl<'a> Init<'a> {
    pub fn new(client: &'a mut Client, reader: &'a mut StreamReader<'a>) -> Self {
        let mut packet = client::InitInit::default();
        packet.deserialize(reader);
        Self { client, packet }
    }
    pub fn handle_packet(&mut self) -> std::io::Result<()> {
        let mut reply = server::InitInit::new();
        reply.reply_code = InitReply::OK;

        // TODO: check for version.. bans.. etc.
        self.client.init_new_sequence();
        let init_ok = self.create_response_ok();
        reply.reply = Box::new(init_ok);
        self.client.state = ClientState::Initialized;
        self.client
            .send(Family::Init, Action::Init, reply.serialize())?;

        Ok(())
    }

    fn create_response_ok(&self) -> server::InitOk {
        let mut init_ok = server::InitOk::new();
        init_ok.challenge_response = stupid_hash(self.packet.challenge);
        init_ok.player_id = self.client.player_id;

        let sequence_bytes = self.client.get_init_sequence_bytes();
        init_ok.sequence_bytes = [sequence_bytes.0 as EOByte, sequence_bytes.1];
        init_ok.encoding_multiples = [
            self.client.processor.decode_multiple,
            self.client.processor.encode_multiple,
        ];
        init_ok
    }
}
