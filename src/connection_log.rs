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
        self.entries.get(ip).map(|entry| entry.last_connect)
    }

    pub fn len(&self) -> i32 {
        self.entries.values().map(|entry| entry.connections).sum()
    }

    pub fn add_connection(&mut self, ip: &str) {
        if let Some(entry) = self.entries.get_mut(ip) {
            entry.connections += 1;
            entry.last_connect = Utc::now();
        } else {
            self.entries.insert(ip.to_string(), Entry::new());
        }
    }

    pub fn remove_connection(&mut self, ip: &str) {
        if let Some(entry) = self.entries.get_mut(ip) {
            if entry.connections > 1 {
                entry.connections -= 1;
            } else {
                self.entries.remove(ip);
            }
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
