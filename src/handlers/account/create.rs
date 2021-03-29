use diesel::MysqlConnection;

use crate::{client::Client, services::account::{Creator, Validator}, settings::Settings};

use eo::{
    data::{Serializeable, StreamReader},
    net::packets::client,
    net::{packets::server, AccountReply, Action, Family},
};

pub struct Create<'a> {
    client: &'a mut Client,
    db: &'a MysqlConnection,
    settings: &'a Settings,
    packet: client::account::Create,
}

impl<'a> Create<'a> {
    pub fn new(
        client: &'a mut Client,
        reader: &'a mut StreamReader<'a>,
        db: &'a MysqlConnection,
        settings: &'a Settings
    ) -> Self {
        let mut packet = client::account::Create::new();
        packet.deserialize(reader);
        Self { client, packet, db, settings }
    }
    pub fn handle_packet(&mut self) -> std::io::Result<()> {
        let mut reply = server::account::Reply::new();
        let mut valid = true;

        {
            let validator = Validator::new(&self.packet.name, self.db);
            if !validator.name_is_valid() {
                reply.reply = AccountReply::NotApproved;
                reply.message = "NO".to_string();
                valid = false;
            } else if validator.account_exists() {
                reply.reply = AccountReply::Exists;
                reply.message = "NO".to_string();
                valid = false;
            }
        }

        if valid {
            let creator = Creator::new(&self.packet, self.db, &self.client.ip_address, &self.settings.server.password_salt);
            match creator.create_account() {
                Ok(_) => info!("Account created: {}", self.packet.name),
                _ => error!("Failed to create account!"),
            };
            reply.reply = AccountReply::Created;
            reply.message = "OK".to_string();
        }

        self.client
            .send(Family::Account, Action::Reply, reply.serialize())
    }
}
