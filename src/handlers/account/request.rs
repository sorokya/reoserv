use diesel::MysqlConnection;

use crate::{client::Client, services::account::Validator};

use eo::{
    data::{EOChar, Serializeable, StreamReader},
    net::packets::client,
    net::{packets::server, AccountReply, Action, Family},
};

pub struct Request<'a> {
    client: &'a mut Client,
    db: &'a MysqlConnection,
    packet: client::account::Request,
}

impl<'a> Request<'a> {
    pub fn new(
        client: &'a mut Client,
        reader: &'a mut StreamReader<'a>,
        db: &'a MysqlConnection,
    ) -> Self {
        let mut packet = client::account::Request::new();
        packet.deserialize(reader);
        Self { client, packet, db }
    }
    pub fn handle_packet(&mut self) -> std::io::Result<()> {
        let mut reply = server::account::Reply::new();
        let validator =  Validator::new(&self.packet.name, self.db);

        if !validator.name_is_valid() {
            reply.reply = AccountReply::NotApproved;
            reply.message = "NO".to_string();
        } else if validator.account_exists() {
            reply.reply = AccountReply::Exists;
            reply.message = "NO".to_string();
        } else {
            reply.reply = AccountReply::Continue;
            if self.client.sequence_start > 240 {
                self.client.account_reply_new_sequence();
            }
            reply.sequence = self.client.sequence_start as EOChar;
            reply.message = "OK".to_string();
        }

        self.client
            .send(Family::Account, Action::Reply, reply.serialize())
    }
}
