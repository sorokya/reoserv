use tokio::sync::oneshot;

use crate::db::Row;

pub enum Command {
    Execute(String, oneshot::Sender<anyhow::Result<()>>),
    Query(String, oneshot::Sender<anyhow::Result<Vec<Row>>>),
}
