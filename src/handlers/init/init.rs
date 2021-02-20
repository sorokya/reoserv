use crate::client::Client;
use eo::{
    data::{Serializeable, StreamReader},
    net::{
        packets::{client, server},
        stupid_hash, Action, Family, InitReply,
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
        let init_ok = self.create_response_ok();
        self.client
            .processor
            .set_multiples(init_ok.encoding_multiples[0], init_ok.encoding_multiples[1]);
        reply.reply = Box::new(init_ok);

        self.client
            .send(Family::Init, Action::Init, reply.serialize())?;

        Ok(())
    }

    fn create_response_ok(&self) -> server::InitOk {
        let mut init_ok = server::InitOk::new();
        init_ok.challenge_response = stupid_hash(self.packet.challenge);
        init_ok.player_id = 1; // TODO: actual player id
        init_ok.sequence_bytes = [6, 12]; // TODO: actual sequence generator
        init_ok.encoding_multiples = [5, 10]; // TODO: actual encode multiples generator
        init_ok
    }
}
