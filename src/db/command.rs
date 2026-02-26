use tokio::sync::oneshot;

use crate::db::{DbRequest, Row};

pub enum Command {
    Execute(DbRequest, oneshot::Sender<anyhow::Result<()>>),
    Query(DbRequest, oneshot::Sender<anyhow::Result<Vec<Row>>>),
    StartTransaction(oneshot::Sender<anyhow::Result<()>>),
    CommitTransaction(oneshot::Sender<anyhow::Result<()>>),
    #[allow(dead_code)]
    RollbackTransaction(oneshot::Sender<anyhow::Result<()>>),
    GetLastInsertId(oneshot::Sender<Option<u64>>),
}
