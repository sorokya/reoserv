use tokio::sync::mpsc::{self, UnboundedSender};

use crate::db::{Command, Db};

#[derive(Debug, Clone)]
pub struct DbHandle {
    tx: UnboundedSender<Command>,
}

impl DbHandle {
    pub fn new() -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let db = Db::new(rx);
        tokio::spawn(run_db(db));
        Self { tx }
    }

    pub async fn execute(&self, query: String) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::Execute(query, resp_tx));
        resp_rx.await?
    }

    pub async fn query(&self, query: String) -> anyhow::Result<Vec<crate::db::Row>> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::Query(query, resp_tx));
        resp_rx.await?
    }
}

async fn run_db(mut db: Db) {
    loop {
        if let Some(command) = db.rx.recv().await {
            db.handle_command(command).await;
        }
    }
}
