use std::collections::HashMap;

use chrono::{DateTime, Utc};

#[derive(Debug)]
pub struct ConnectionLog {
    entries: HashMap<String, Entry>,
}

impl ConnectionLog {
    pub fn new() -> Self {
        Self {
            entries: HashMap::new(),
        }
    }

    pub fn get_num_of_connections(&self, ip: &str) -> i32 {
        match self.entries.get(ip) {
            Some(entry) => entry.connections,
            None => 0,
        }
    }

    pub fn get_last_connect(&self, ip: &str) -> Option<DateTime<Utc>> {
        match self.entries.get(ip) {
            Some(entry) => Some(entry.last_connect),
            None => None,
        }
    }

    pub fn len(&self) -> i32 {
        self.entries
            .iter()
            .map(|(_, entry)| entry.connections)
            .sum()
    }

    pub fn add_connection(&mut self, ip: &str) {
        if let Some(entry) = self.entries.get_mut(ip) {
            entry.connections += 1;
            entry.last_connect = Utc::now();
        } else {
            self.entries.insert(ip.to_string(), Entry::new());
        }
    }
}

#[derive(Debug)]
pub struct Entry {
    pub connections: i32,
    pub last_connect: DateTime<Utc>,
}

impl Entry {
    pub fn new() -> Self {
        Self {
            connections: 1,
            last_connect: Utc::now(),
        }
    }
}
