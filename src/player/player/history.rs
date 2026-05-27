use crate::db::insert_params;

use super::{super::Action, Player};

impl Player {
    pub fn record_action(&self, action: Action) {
        self.record_action_with_details(action, "");
    }

    pub fn record_action_with_details(&self, action: Action, details: &str) {
        let db = self.db.clone();
        let account_id = self.account_id;
        let character_id = self.character_id;
        let ip = self.ip.clone();
        let details = details.to_string();
        tokio::spawn(async move {
            if let Err(e) = db.execute(
                &insert_params(
                    "INSERT INTO history (account_id, character_id, action_id, ip, details) VALUES (:account_id, :character_id, :action_id, :ip, :details)",
                    &[
                        ("account_id", &account_id),
                        ("character_id", &character_id),
                        ("action_id", &(action as i32)),
                        ("ip", &ip),
                        ("details", &details),
                    ],
                )).await {
                    tracing::error!("Failed to record history action: {}", e);
                }
        });
    }
}
