use crate::models::Account;
use crate::schema::accounts::dsl::*;
use diesel::{prelude::*, MysqlConnection};
use regex::Regex;

pub struct Validator<'a> {
    username: &'a str,
    db: &'a MysqlConnection,
}

impl<'a> Validator<'a> {
    pub fn new(username: &'a str, db: &'a MysqlConnection) -> Self {
        Self { username, db }
    }
    pub fn name_is_valid(&self) -> bool {
        let regex = Regex::new(r"^[a-z0-9]*$").expect("Failed to create regex");
        self.username.len() <= 16 && regex.is_match(&self.username)
    }

    pub fn account_exists(&self) -> bool {
        let results = accounts
            .filter(name.eq(self.username))
            .limit(1)
            .load::<Account>(self.db)
            .expect("Error loading account");

        results.len() > 0
    }
}
