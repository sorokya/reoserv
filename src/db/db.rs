use tokio::sync::mpsc::UnboundedReceiver;

use crate::db::Connection;

use super::Command;

#[derive(Debug)]
pub struct Db {
    pub rx: UnboundedReceiver<Command>,
    connection: Connection,
    last_insert_id: Option<u64>,
}

mod execute;
mod query;

impl Db {
    pub fn new(rx: UnboundedReceiver<Command>, connection: Connection) -> Self {
        Self {
            rx,
            connection,
            last_insert_id: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Execute(query, resp_tx) => {
                let result = self.execute(query).await;
                let _ = resp_tx.send(result);
            }
            Command::Query(query, resp_tx) => {
                let result = self.query(query).await;
                let _ = resp_tx.send(result);
            }
            Command::GetLastInsertId(resp_tx) => {
                let _ = resp_tx.send(self.last_insert_id);
            }
            Command::StartTransaction(resp_tx) => {
                let result = match self.connection {
                    Connection::Mysql(_) => self.execute("START TRANSACTION".to_string()).await,
                    Connection::Sqlite(_) => self.execute("BEGIN TRANSACTION".to_string()).await,
                };
                let _ = resp_tx.send(result);
            }
            Command::CommitTransaction(resp_tx) => {
                let result = self.execute("COMMIT".to_string()).await;
                let _ = resp_tx.send(result);
            }
            Command::RollbackTransaction(resp_tx) => {
                let result = self.execute("ROLLBACK".to_string()).await;
                let _ = resp_tx.send(result);
            }
        }
    }
}
