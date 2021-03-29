use std::num::ParseIntError;

use crate::schema::accounts::dsl::*;
use chrono::prelude::*;
use diesel::{associations::HasTable, prelude::*, MysqlConnection};
use eo::net::packets::client::account::Create;
use sha2::{Digest, Sha256};

pub struct Creator<'a> {
    packet: &'a Create,
    db: &'a MysqlConnection,
    client_ip: &'a str,
    salt: &'a str,
}

impl<'a> Creator<'a> {
    pub fn new(packet: &'a Create, db: &'a MysqlConnection, client_ip: &'a str, salt: &'a str) -> Self {
        Self {
            packet,
            db,
            client_ip,
            salt,
        }
    }
    pub fn create_account(&self) -> Result<bool, ParseIntError> {
        let hash_input = format!(
            "{}{}{}",
            self.salt.to_string(), self.packet.name, self.packet.password
        );
        let hash = Sha256::digest(hash_input.as_bytes());
        let record = (
            name.eq(self.packet.name.to_string()),
            password_hash.eq(format!("{:x}", hash)),
            real_name.eq(self.packet.fullname.to_string()),
            location.eq(self.packet.location.to_string()),
            email.eq(self.packet.email.to_string()),
            computer.eq(self.packet.computer.to_string()),
            hdid.eq(match self.packet.hdid.parse::<i32>() {
                Ok(parsed) => parsed,
                Err(e) => {
                    return Err(e);
                }
            }),
            register_ip.eq(self.client_ip.to_string()),
            created_on.eq(Utc::now().naive_utc()),
        );

        diesel::insert_into(accounts::table())
            .values(&record)
            .execute(self.db)
            .expect("Error saving account");

        Ok(true)
    }
}
