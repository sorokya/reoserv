use tokio::sync::mpsc::UnboundedReceiver;

use crate::db::Connection;

use super::{Command, DbRequest};

#[derive(Debug)]
pub struct Db {
    pub rx: UnboundedReceiver<Command>,
    connection: Connection,
    transaction_active: bool,
    last_insert_id: Option<u64>,
}

mod execute;
mod query;
mod transaction;

impl Db {
    pub fn new(rx: UnboundedReceiver<Command>, connection: Connection) -> Self {
        Self {
            rx,
            connection,
            transaction_active: false,
            last_insert_id: None,
        }
    }

    pub async fn handle_command(&mut self, command: Command) {
        match command {
            Command::Execute(query, resp_tx) => {
                let result = match query {
                    DbRequest::Raw(query) => self.execute(&query).await,
                    DbRequest::Prepared(query) => self.execute_prepared(&query).await,
                };
                let _ = resp_tx.send(result);
            }
            Command::Query(query, resp_tx) => {
                let result = match query {
                    DbRequest::Raw(query) => self.query(&query).await,
                    DbRequest::Prepared(query) => self.query_prepared(&query).await,
                };
                let _ = resp_tx.send(result);
            }
            Command::GetLastInsertId(resp_tx) => {
                let _ = resp_tx.send(self.last_insert_id);
            }
            Command::StartTransaction(resp_tx) => {
                let result = self.start_transaction().await;
                let _ = resp_tx.send(result);
            }
            Command::CommitTransaction(resp_tx) => {
                let result = self.commit_transaction().await;
                let _ = resp_tx.send(result);
            }
            Command::RollbackTransaction(resp_tx) => {
                let result = self.rollback_transaction().await;
                let _ = resp_tx.send(result);
            }
        }
    }

    pub(super) async fn with_transaction_rollback<T>(
        &mut self,
        result: anyhow::Result<T>,
    ) -> anyhow::Result<T> {
        match result {
            Ok(value) => Ok(value),
            Err(error) => {
                self.rollback_if_transaction_active().await;
                Err(error)
            }
        }
    }

    pub(super) async fn rollback_if_transaction_active(&mut self) {
        if self.transaction_active {
            if let Err(error) = self.execute_inner("ROLLBACK").await {
                error!("Failed to rollback transaction: {}", error);
            }
            self.transaction_active = false;
        }
    }

    pub(super) async fn reconnect_mysql(&mut self) -> anyhow::Result<()> {
        if let crate::db::Connection::Mysql(ref mut mysql) = self.connection {
            warn!("MySQL connection closed, reconnecting...");
            mysql.conn = mysql_async::Conn::from_url(mysql.url.clone()).await?;
        }
        Ok(())
    }
}

pub(super) fn is_mysql_connection_closed(e: &anyhow::Error) -> bool {
    if let Some(mysql_err) = e.downcast_ref::<mysql_async::Error>() {
        match mysql_err {
            mysql_async::Error::Driver(mysql_async::DriverError::ConnectionClosed) => true,
            mysql_async::Error::Io(io_err) => {
                let msg = io_err.to_string().to_lowercase();
                msg.contains("connection closed")
                    || msg.contains("connection to the server is closed")
            }
            _ => false,
        }
    } else {
        false
    }
}

#[cfg(test)]
mod tests {
    use crate::db::{Connection, Db};

    #[test]
    fn test_db_creation() {
        let (_, rx) = tokio::sync::mpsc::unbounded_channel();
        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let db = Db::new(rx, connection);
        assert!(!db.transaction_active);
        assert!(db.last_insert_id.is_none());
    }

    #[tokio::test]
    async fn test_db_transaction_rollback() {
        let (_, rx) = tokio::sync::mpsc::unbounded_channel();
        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let mut db = Db::new(rx, connection);
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();
        db.execute("INSERT INTO test (id) VALUES (1)")
            .await
            .unwrap();
        db.start_transaction().await.unwrap();
        db.execute("INSERT INTO test (id) VALUES (2)")
            .await
            .unwrap();
        db.rollback_transaction().await.unwrap();

        let row = db.query("SELECT COUNT(1) FROM test").await.unwrap();
        assert_eq!(row[0].columns[0], crate::db::row::SqlValue::Int(1));
        assert!(!db.transaction_active);
    }

    #[tokio::test]
    async fn test_db_transaction_commit() {
        let (_, rx) = tokio::sync::mpsc::unbounded_channel();
        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let mut db = Db::new(rx, connection);
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();
        db.execute("INSERT INTO test (id) VALUES (1)")
            .await
            .unwrap();
        db.start_transaction().await.unwrap();
        db.execute("INSERT INTO test (id) VALUES (2)")
            .await
            .unwrap();
        db.commit_transaction().await.unwrap();

        let row = db.query("SELECT COUNT(1) FROM test").await.unwrap();
        assert_eq!(row[0].columns[0], crate::db::row::SqlValue::Int(2));
        assert!(!db.transaction_active);
    }

    #[tokio::test]
    async fn test_db_transaction_error_handling() {
        let (_, rx) = tokio::sync::mpsc::unbounded_channel();
        let connection = Connection::Sqlite(rusqlite::Connection::open_in_memory().unwrap());
        let mut db = Db::new(rx, connection);
        db.execute("CREATE TABLE test (id INTEGER PRIMARY KEY)")
            .await
            .unwrap();
        db.start_transaction().await.unwrap();
        let result = db.execute("INSERT INTO test (id) VALUES ('invalid')").await;
        assert!(result.is_err());
        assert!(!db.transaction_active);

        let row = db.query("SELECT COUNT(1) FROM test").await.unwrap();
        assert_eq!(row[0].columns[0], crate::db::row::SqlValue::Int(0));
    }
}
