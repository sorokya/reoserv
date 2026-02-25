use tokio::sync::mpsc::{self, UnboundedSender};

use super::{Command, Connection, Db};

#[derive(Debug, Clone)]
pub struct DbHandle {
    tx: UnboundedSender<Command>,
}

impl DbHandle {
    pub fn new(connection: Connection) -> Self {
        let (tx, rx) = mpsc::unbounded_channel();
        let db = Db::new(rx, connection);
        tokio::spawn(run_db(db));
        Self { tx }
    }

    pub async fn execute(&self, query: &str) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::Execute(query.to_string(), resp_tx));
        resp_rx.await?
    }

    pub async fn query(&self, query: &str) -> anyhow::Result<Vec<crate::db::Row>> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::Query(query.to_string(), resp_tx));
        resp_rx.await?
    }

    pub async fn query_map<T, F>(&self, query: &str, map_fn: F) -> anyhow::Result<Vec<T>>
    where
        F: Fn(crate::db::Row) -> T,
    {
        let rows = self.query(query).await?;
        Ok(rows.into_iter().map(map_fn).collect())
    }

    pub async fn query_one(&self, query: &str) -> anyhow::Result<Option<crate::db::Row>> {
        let rows = self.query(query).await?;
        Ok(rows.into_iter().next())
    }

    pub async fn query_string(&self, query: &str) -> anyhow::Result<Option<String>> {
        let row = self.query_one(query).await?;
        Ok(row.and_then(|r| r.get_string(0)))
    }

    pub async fn query_int(&self, query: &str) -> anyhow::Result<Option<i32>> {
        let row = self.query_one(query).await?;
        Ok(row.and_then(|r| r.get_int(0)))
    }

    #[allow(dead_code)]
    pub async fn query_date(&self, query: &str) -> anyhow::Result<Option<chrono::NaiveDateTime>> {
        let row = self.query_one(query).await?;
        Ok(row.and_then(|r| r.get_date(0)))
    }

    pub async fn start_transaction(&self) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::StartTransaction(resp_tx));
        resp_rx.await?
    }

    pub async fn commit_transaction(&self) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::CommitTransaction(resp_tx));
        resp_rx.await?
    }

    pub async fn get_last_insert_id(&self) -> Option<u64> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::GetLastInsertId(resp_tx));
        resp_rx.await.ok().flatten()
    }

    #[allow(dead_code)]
    pub async fn rollback_transaction(&self) -> anyhow::Result<()> {
        let (resp_tx, resp_rx) = tokio::sync::oneshot::channel();
        let _ = self.tx.send(Command::RollbackTransaction(resp_tx));
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
