use tokio::sync::mpsc::UnboundedReceiver;

use super::Command;

#[derive(Debug)]
pub struct Db {
    pub rx: UnboundedReceiver<Command>,
}

mod execute;
mod query;

impl Db {
    pub fn new(rx: UnboundedReceiver<Command>) -> Self {
        Self { rx }
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
        }
    }
}
