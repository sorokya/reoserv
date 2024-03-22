use bytes::Bytes;
use chrono::{DateTime, Utc};
use eolib::protocol::net::{PacketAction, PacketFamily};

#[derive(Debug)]
pub struct PacketLog {
    entries: Vec<Entry>,
}

#[derive(Debug)]
pub struct Entry {
    pub action: PacketAction,
    pub family: PacketFamily,
    pub processed_at: DateTime<Utc>,
}

impl PacketLog {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
        }
    }

    pub fn last_processed(&self, buf: &Bytes) -> Option<DateTime<Utc>> {
        let action = PacketAction::from(buf[0]);
        let family = PacketFamily::from(buf[1]);

        self.entries
            .iter()
            .find(|e| e.action == action && e.family == family)
            .map(|entry| entry.processed_at)
    }

    pub fn add_entry(&mut self, buf: &Bytes) {
        let action = PacketAction::from(buf[0]);
        let family = PacketFamily::from(buf[1]);

        if let Some(entry) = self
            .entries
            .iter_mut()
            .find(|e| e.action == action && e.family == family)
        {
            entry.processed_at = Utc::now();
        } else {
            self.entries.push(Entry {
                action,
                family,
                processed_at: Utc::now(),
            });
        }
    }

    pub fn clean_old_entries(&mut self) {
        let now = Utc::now();
        self.entries
            .retain(|e| now.signed_duration_since(e.processed_at).num_seconds() < 1);
    }
}
